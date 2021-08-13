/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/
use sgx_types::*;

use codec::{Decode, Encode};
use log::*;
use std::collections::HashMap;
use std::slice;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use ws::{Builder, CloseCode, Handler, Message, Result, Sender, Settings};

use polkadex_sgx_primitives::RequestId;
use substratee_worker_primitives::{DirectRequestStatus, RpcResponse, RpcReturnValue};

static WATCHED_LIST: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());
static EID: AtomicPtr<u64> = AtomicPtr::new(0 as *mut sgx_enclave_id_t);
const CONNECTIONS: usize = 10_000; // simultaneous ws connections

extern "C" {
    fn initialize_pool(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;

    fn call_rpc_methods(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        request: *const u8,
        request_len: u32,
        response: *mut u8,
        response_len: u32,
    ) -> sgx_status_t;
}

#[derive(Clone, Debug)]
pub struct DirectWsServerRequest {
    client: Sender,
    request: String,
}

impl DirectWsServerRequest {
    pub fn new(client: Sender, request: String) -> Self {
        Self { client, request }
    }
}

pub fn start_worker_api_direct_server(addr: String, eid: sgx_enclave_id_t) {
    // initialise top pool in enclave
    let init = thread::spawn(move || {
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let result = unsafe { initialize_pool(eid, &mut retval) };

        match result {
            sgx_status_t::SGX_SUCCESS => {
                debug!("[TX-pool init] ECALL success!");
            }
            _ => {
                error!("[TX-pool init] ECALL Enclave Failed {}!", result.as_str());
            }
        }
    });

    // Server WebSocket handler
    struct Server {
        client: Sender,
    }

    // initialize static pointer to eid
    let eid_ptr = Arc::into_raw(Arc::new(eid));
    EID.store(eid_ptr as *mut sgx_enclave_id_t, Ordering::SeqCst);

    impl Handler for Server {
        fn on_message(&mut self, msg: Message) -> Result<()> {
            let request = DirectWsServerRequest::new(self.client.clone(), msg.to_string());
            if handle_direct_invocation_request(request).is_err() {
                error!("direct invocation call was not successful");
            }
            Ok(())
        }

        fn on_close(&mut self, code: CloseCode, reason: &str) {
            debug!(
                "Direct invocation WebSocket closing for ({:?}) {}",
                code, reason
            );
        }
    }

    // Server thread
    info!("Starting direct invocation WebSocket server on {}", addr);
    thread::spawn(move || {
        let settings = Settings {
            max_connections: CONNECTIONS,
            ..Default::default()
        };

        match Builder::new()
            .with_settings(settings)
            .build(|out: Sender| Server { client: out })
            .unwrap()
            .listen(addr.clone())
        {
            Ok(_) => (),
            Err(e) => {
                error!(
                    "error starting worker direct invocation api server on {}: {}",
                    addr, e
                );
            }
        }
    });

    // initialize static pointer to empty HashMap
    let new_map: HashMap<RequestId, WatchingClient> = HashMap::new();
    let pool_ptr = Arc::new(Mutex::new(new_map));
    let ptr = Arc::into_raw(pool_ptr);
    WATCHED_LIST.store(ptr as *mut (), Ordering::SeqCst);

    // ensure top pool is initialised before returning
    init.join().unwrap();
    println!("Successfully initialised top pool");
}

struct WatchingClient {
    client: Sender,
    response: RpcResponse,
}

fn load_watched_list() -> Option<&'static Mutex<HashMap<RequestId, WatchingClient>>> {
    let ptr = WATCHED_LIST.load(Ordering::SeqCst) as *mut Mutex<HashMap<RequestId, WatchingClient>>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &*ptr })
    }
}

pub fn handle_direct_invocation_request(req: DirectWsServerRequest) -> Result<()> {
    info!("Got message '{:?}'. ", req.request);
    let eid = unsafe { *EID.load(Ordering::SeqCst) };
    // forwarding rpc string directly to enclave
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let response_len = 8192;
    let mut response: Vec<u8> = vec![0u8; response_len as usize];

    let msg: Vec<u8> = req.request.as_bytes().to_vec();

    let result = unsafe {
        call_rpc_methods(
            eid,
            &mut retval,
            msg.as_ptr(),
            msg.len() as u32,
            response.as_mut_ptr(),
            response_len,
        )
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {
            debug!("[RPC-Call] ECALL success!");
        }
        _ => {
            error!("[RPC-call] ECALL Enclave Failed {}!", result.as_str());
        }
    }
    let decoded_response = String::from_utf8_lossy(&response).to_string();
    if let Ok(full_rpc_response) =
        serde_json::from_str(&decoded_response) as serde_json::Result<RpcResponse>
    {
        if let Ok(result_of_rpc_response) =
            RpcReturnValue::decode(&mut full_rpc_response.result.as_slice())
        {
            if let DirectRequestStatus::TrustedOperationStatus(_) = result_of_rpc_response.status {
                if result_of_rpc_response.do_watch {
                    // start watching the call with the specific hash

                    if let Ok(request) =
                        RequestId::decode(&mut result_of_rpc_response.value.as_slice())
                    {
                        // Aquire lock on watched list
                        let mutex = load_watched_list().unwrap();
                        let mut watch_list: MutexGuard<HashMap<u128, WatchingClient>> =
                            mutex.lock().unwrap();

                        // create new key and value entries to store
                        let new_client = WatchingClient {
                            client: req.client.clone(),
                            response: RpcResponse {
                                result: result_of_rpc_response.encode(),
                                jsonrpc: full_rpc_response.jsonrpc.clone(),
                                id: full_rpc_response.id,
                            },
                        };
                        // save in watch list
                        watch_list.insert(request, new_client);
                    }
                }
            }
        }
        return req
            .client
            .send(serde_json::to_string(&full_rpc_response).unwrap());
    }
    // could not decode rpcresponse - maybe a String as return value?
    req.client.send(decoded_response)
}

#[no_mangle]
pub unsafe extern "C" fn ocall_update_status_event(
    _hash_encoded: *const u8,
    _hash_size: u32,
    _status_update_encoded: *const u8,
    _status_size: u32,
) -> sgx_status_t {
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ocall_send_response_with_uuid(
    request_id_encoded: *const u8,
    request_id_size: u32,
    uuid_encoded: *const u8,
    uuid_size: u32,
) -> sgx_status_t {
    let mut request_id_slice = slice::from_raw_parts(request_id_encoded, request_id_size as usize);
    let uuid_slice = slice::from_raw_parts(uuid_encoded, uuid_size as usize);
    if let Ok(request_id) = u128::decode(&mut request_id_slice) {
        let mutex = if let Some(mutex) = load_watched_list() {
            mutex
        } else {
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        };
        let mut guard = mutex.lock().unwrap();
        let submitted = DirectRequestStatus::Ok;
        let result = RpcReturnValue::new(uuid_slice.to_vec(), false, submitted);

        if let Some(client_response) = guard.get_mut(&request_id) {
            let mut response = &mut client_response.response;
            response.result = result.encode();
            client_response
                .client
                .send(serde_json::to_string(uuid_slice).unwrap())
                .unwrap();

            client_response.client.close(CloseCode::Normal).unwrap();
        } else {
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
        guard.remove(&request_id);
    }
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ocall_send_status(
    _hash_encoded: *const u8,
    _hash_size: u32,
    _status_encoded: *const u8,
    _status_size: u32,
) -> sgx_status_t {
    sgx_status_t::SGX_SUCCESS
}
