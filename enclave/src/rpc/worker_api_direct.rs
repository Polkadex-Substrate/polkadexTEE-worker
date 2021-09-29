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

pub extern crate alloc;
use crate::rpc::polkadex_rpc_gateway::PolkadexRpcGateway;
use crate::rpc::return_value_encoding::compute_encoded_return_error;
use crate::rpc::trusted_operation_verifier::TrustedOperationVerifier;
use crate::rpc::{
    api::SideChainApi, basic_pool::BasicPool, io_handler_extensions, rpc_call_encoder::RpcCall,
    rpc_cancel_order::RpcCancelOrder, rpc_get_balance::RpcGetBalance, rpc_nonce::RpcNonce,
    rpc_place_order::RpcPlaceOrder, rpc_withdraw::RpcWithdraw,
};
use crate::rsa3072;
use crate::top_pool::pool::Options as PoolOptions;
use crate::utils::write_slice_and_whitespace_pad;
use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    slice::{from_raw_parts, from_raw_parts_mut},
    str,
    string::String,
    vec::Vec,
};
use base58::FromBase58;
use chain_relay::Block;
use codec::{Decode, Encode};
use core::result::Result;
use jsonrpc_core::*;
use log::*;
use serde_json::*;
use sgx_types::*;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex,
};

use self::serde_json::*;
use crate::rpc::{
    api::SideChainApi,
    author::{Author, AuthorApi},
    basic_pool::BasicPool,
};
use crate::top_pool::pool::Options as PoolOptions;
use base58::FromBase58;
use chain_relay::Block;
use jsonrpc_core::{futures::executor, Error as RpcError, *};
use log::*;
use sp_core::H256 as Hash;
use substratee_node_primitives::Request;
use substratee_stf::ShardIdentifier;
use substratee_worker_primitives::{
    block::SignedBlock, DirectRequestStatus, RpcReturnValue, TrustedOperationStatus,
};

extern "C" {
    pub fn ocall_update_status_event(
        ret_val: *mut sgx_status_t,
        hash_encoded: *const u8,
        hash_size: u32,
        status_update_encoded: *const u8,
        status_size: u32,
    ) -> sgx_status_t;
    pub fn ocall_send_status(
        ret_val: *mut sgx_status_t,
        hash_encoded: *const u8,
        hash_size: u32,
        status_update_encoded: *const u8,
        status_size: u32,
    ) -> sgx_status_t;
    pub fn ocall_send_response_with_uuid(
        ret_val: *mut sgx_status_t,
        request_id_encoded: *const u8,
        request_id_size: u32,
        uuid_encoded: *const u8,
        uuid_size: u32,
    ) -> sgx_status_t;
}
use crate::{ocall::rpc_ocall::EnclaveRpcOCall, rsa3072, utils::write_slice_and_whitespace_pad};

static GLOBAL_TX_POOL: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[no_mangle]
// initialise tx pool and store within static atomic pointer
pub unsafe extern "C" fn initialize_pool() -> sgx_status_t {
    let api = Arc::new(SideChainApi::new());
    let tx_pool = BasicPool::create(PoolOptions::default(), api);
    let pool_ptr = Arc::new(SgxMutex::<
        BasicPool<SideChainApi<Block>, Block, EnclaveRpcOCall>,
    >::new(tx_pool));
    let ptr = Arc::into_raw(pool_ptr);
    GLOBAL_TX_POOL.store(ptr as *mut (), Ordering::SeqCst);

    sgx_status_t::SGX_SUCCESS
}

pub fn load_top_pool(
) -> Option<&'static SgxMutex<BasicPool<SideChainApi<Block>, Block, EnclaveRpcOCall>>> {
    let ptr = GLOBAL_TX_POOL.load(Ordering::SeqCst)
        as *mut SgxMutex<BasicPool<SideChainApi<Block>, Block, EnclaveRpcOCall>>;
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &*ptr })
    }
}

// converts the rpc methods vector to a string and adds commas and brackets for readability
fn convert_vec_to_string(vec_methods: Vec<&str>) -> String {
    let mut method_string = String::new();
    for i in 0..vec_methods.len() {
        method_string.push_str(vec_methods[i]);
        if vec_methods.len() > (i + 1) {
            method_string.push_str(", ");
        }
    }
    format!("methods: [{}]", method_string)
}

// converts the rpc methods vector to a string and adds commas and brackets for readability
#[allow(unused)]
fn decode_shard_from_base58(shard_base58: String) -> Result<ShardIdentifier, String> {
    let shard_vec = match shard_base58.from_base58() {
        Ok(vec) => vec,
        Err(_) => return Err("Invalid base58 format of shard id".to_owned()),
    };
    let shard = match ShardIdentifier::decode(&mut shard_vec.as_slice()) {
        Ok(hash) => hash,
        Err(_) => return Err("Shard ID is not of type H256".to_owned()),
    };
    Ok(shard)
}

fn init_io_handler() -> IoHandler {
    let mut io = IoHandler::new();

    // PLACE ORDER
    io.add_sync_method(
        &RpcPlaceOrder::name(),
        RpcPlaceOrder::new(
            Box::new(TrustedOperationVerifier {}),
            Box::new(PolkadexRpcGateway {}),
        ),
    );

    // CANCEL ORDER
    io.add_sync_method(
        &RpcCancelOrder::name(),
        RpcCancelOrder::new(
            Box::new(TrustedOperationVerifier {}),
            Box::new(PolkadexRpcGateway {}),
        ),
    );

    // WITHDRAW
    io.add_sync_method(
        &RpcWithdraw::name(),
        RpcWithdraw::new(
            Box::new(TrustedOperationVerifier {}),
            Box::new(PolkadexRpcGateway {}),
        ),
    );

    // GET BALANCE
    io.add_sync_method(
        &RpcGetBalance::name(),
        RpcGetBalance::new(
            Box::new(TrustedOperationVerifier {}),
            Box::new(PolkadexRpcGateway {}),
        ),
    );

    // GET BALANCE
    io.add_sync_method(
        &RpcNonce::name(),
        RpcNonce::new(
            Box::new(TrustedOperationVerifier {}),
            Box::new(PolkadexRpcGateway {}),
        ),
    );

    // author_getShieldingKey
    let rsa_pubkey_name: &str = "author_getShieldingKey";
    io.add_sync_method(rsa_pubkey_name, move |_: Params| {
        let rsa_pubkey = match rsa3072::unseal_pubkey() {
            Ok(key) => key,
            Err(status) => {
                let error_msg: String = format!("Could not get rsa pubkey due to: {}", status);
                return Ok(json!(compute_encoded_return_error(&error_msg)));
            }
        };

        let rsa_pubkey_json = match serde_json::to_string(&rsa_pubkey) {
            Ok(k) => k,
            Err(x) => {
                let error_msg: String = format!(
                    "[Enclave] can't serialize rsa_pubkey {:?} {}",
                    rsa_pubkey, x
                );
                return Ok(json!(compute_encoded_return_error(&error_msg)));
            }
        };
        let json_value =
            RpcReturnValue::new(rsa_pubkey_json.encode(), false, DirectRequestStatus::Ok);
        Ok(json!(json_value.encode()))
    });

    // returns all rpcs methods
    let rpc_methods_string: String = io_handler_extensions::get_all_rpc_methods_string(&io);
    io.add_sync_method("rpc_methods", move |_: Params| {
        Ok(Value::String(rpc_methods_string.to_owned()))
    });

    io
}

#[no_mangle]
pub unsafe extern "C" fn call_rpc_methods(
    request: *const u8,
    request_len: u32,
    response: *mut u8,
    response_len: u32,
) -> sgx_status_t {
    // init
    let io = init_io_handler();
    // get request string
    let req: Vec<u8> = from_raw_parts(request, request_len as usize).to_vec();
    let request_string = match str::from_utf8(&req) {
        Ok(req) => req,
        Err(e) => {
            error!("Decoding Header failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };
    // Rpc Response String
    let response_string = io.handle_request_sync(request_string).unwrap();
    debug!("Response String: {:?}", response_string);
    // update response outside of enclave
    let response_slice = from_raw_parts_mut(response, response_len as usize);
    write_slice_and_whitespace_pad(response_slice, response_string.as_bytes().to_vec());
    sgx_status_t::SGX_SUCCESS
}

pub mod tests {
    use super::{alloc::string::ToString, init_io_handler};
    use std::string::String;

    fn rpc_response<T: ToString>(result: T) -> String {
        format!(
            r#"{{"jsonrpc":"2.0","result":{},"id":1}}"#,
            result.to_string()
        )
    }

    pub fn sidechain_import_block_is_ok() {
        let io = init_io_handler();
        let enclave_req = r#"{"jsonrpc":"2.0","method":"sidechain_importBlock","params":[4,0,0,0,0,0,0,0,0,228,0,145,188,97,251,138,131,108,29,6,107,10,152,67,29,148,190,114,167,223,169,197,163,93,228,76,169,171,80,15,209,101,11,211,96,0,0,0,0,83,52,167,255,37,229,185,231,38,66,122,3,55,139,5,190,125,85,94,177,190,99,22,149,92,97,154,30,142,89,24,144,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,136,220,52,23,213,5,142,196,180,80,62,12,18,234,26,10,137,190,32,15,233,137,34,66,61,67,52,1,79,166,176,238,0,0,0,175,124,84,84,32,238,162,224,130,203,26,66,7,121,44,59,196,200,100,31,173,226,165,106,187,135,223,149,30,46,191,95,116,203,205,102,100,85,82,74,158,197,166,218,181,130,119,127,162,134,227,129,118,85,123,76,21,113,90,1,160,77,110,15],"id":1}"#;

        let response_string = io.handle_request_sync(enclave_req).unwrap();

        assert_eq!(response_string, rpc_response("\"ok\""));
    }

    pub fn sidechain_import_block_returns_invalid_param_err() {
        let io = init_io_handler();
        let enclave_req = r#"{"jsonrpc":"2.0","method":"sidechain_importBlock","params":["SophisticatedInvalidParam"],"id":1}"#;

        let response_string = io.handle_request_sync(enclave_req).unwrap();

        let err_msg = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: invalid type: string \"SophisticatedInvalidParam\", expected u8."},"id":1}"#;
        assert_eq!(response_string, err_msg);
    }

    pub fn sidechain_import_block_returns_decode_err() {
        let io = init_io_handler();
        let enclave_req =
            r#"{"jsonrpc":"2.0","method":"sidechain_importBlock","params":[2],"id":1}"#;

        let response_string = io.handle_request_sync(enclave_req).unwrap();

        let err_msg = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid parameters: Could not decode Vec<SignedBlock>","data":"[2]"},"id":1}"#;
        assert_eq!(response_string, err_msg);
    }
}

pub fn send_uuid(request_id: u128, uuid: Vec<u8>) -> Result<(), String> {
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let request_encoded = request_id.encode();

    let res = unsafe {
        ocall_send_response_with_uuid(
            &mut rt as *mut sgx_status_t,
            request_encoded.as_ptr(),
            request_encoded.len() as u32,
            uuid.as_ptr(),
            uuid.len() as u32,
        )
    };

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(String::from("rt not successful"));
    }

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(String::from("res not successful"));
    }

    Ok(())
}
