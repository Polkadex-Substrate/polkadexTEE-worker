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
use alloc::{string::String, vec::Vec};

use codec::{Decode, Encode};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::*;
use polkadex_sgx_primitives::types::DirectRequest;
use serde_json::*;
use substratee_worker_primitives::DirectRequestStatus;

use crate::rpc::return_value_encoding::{
    compute_encoded_return_error, compute_encoded_return_value,
};

type RpcMethodImpl<'a, T> = &'a dyn Fn(DirectRequest) -> RpcResult<(T, bool, DirectRequestStatus)>;

pub trait RpcCallEncoder {
    fn call<T: Encode>(
        params: Params,
        method_impl: RpcMethodImpl<T>,
    ) -> BoxFuture<RpcResult<Value>>;
}

pub struct JsonRpcCallEncoder {}

impl JsonRpcCallEncoder {
    fn handle_request<T: Encode>(
        request: DirectRequest,
        method_impl: RpcMethodImpl<T>,
    ) -> RpcResult<Value> {
        match method_impl(request) {
            Ok(result) => Ok(json!(compute_encoded_return_value(
                result.0, result.1, result.2
            ))),
            Err(e) => Ok(json!(compute_encoded_return_error(&format!(
                "RPC call failed: {}",
                e
            )))),
        }
    }
}

impl RpcCallEncoder for JsonRpcCallEncoder {
    fn call<T: Encode>(
        params: Params,
        method_impl: RpcMethodImpl<T>,
    ) -> BoxFuture<RpcResult<Value>> {
        match params.parse::<Vec<u8>>() {
            Ok(encoded_params) => match DirectRequest::decode(&mut encoded_params.as_slice()) {
                Ok(request) => {
                    JsonRpcCallEncoder::handle_request(request, method_impl).into_future()
                }

                Err(_) => Ok(json!(compute_encoded_return_error(
                    "Could not decode request"
                )))
                .into_future(),
            },
            Err(e) => {
                let error_msg: String = format!("Could not submit trusted call due to: {}", e);
                Ok(json!(compute_encoded_return_error(&error_msg))).into_future()
            }
        }
    }
}

pub mod tests {

    use super::*;
    use jsonrpc_core::futures::executor::block_on;

    pub struct RpcCallEncoderMock {}

    impl RpcCallEncoder for RpcCallEncoderMock {
        fn call<T: Encode>(
            _params: Params,
            _method_impl: RpcMethodImpl<T>,
        ) -> BoxFuture<RpcResult<Value>> {
            Ok(json!(compute_encoded_return_error("encoded successfully"))).into_future()
        }
    }

    pub fn test_encoding_none_params_returns_ok() {
        let expected_do_watch = true;

        let result = block_on(JsonRpcCallEncoder::call(
            Params::None,
            &|_request: DirectRequest| Ok(("message", expected_do_watch, DirectRequestStatus::Ok)),
        ))
        .unwrap();

        if let Value::Array(values) = result {
            assert!(!values.is_empty());
        } else {
            assert!(false, "result did not match a Value::Array as expected");
        }

        //let outer_message = str::from_utf8(&result).unwrap();
        //debug!("{:#?}", outer_message)

        // let rpc_return_value = RpcReturnValue::decode(&mut result).unwrap();
        // let inner_message = str::from_utf8(&rpc_return_value.value).unwrap();

        // assert_eq!(rpc_return_value.status, request_status);
        //assert_eq!(rpc_return_value.do_watch, expected_do_watch);

        //debug!("{:#?}", result);
        //debug!("{:#?}", inner_message);
        //debug!(result.decode());
    }
}
