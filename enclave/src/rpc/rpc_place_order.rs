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

use alloc::str;

use core::marker::{Send, Sync};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::*;

use substratee_node_primitives::Request;
use substratee_worker_primitives::DirectRequestStatus;

use crate::rpc::rpc_call_encoder::RpcCallEncoder;

/// RPC call structure for 'place order'
pub struct RpcPlaceOrder<E: RpcCallEncoder + Send + Sync + 'static> {
    call_encoder: E,
}

impl<E: RpcCallEncoder + Send + Sync + 'static> RpcPlaceOrder<E> {
    pub fn method_name() -> &'static str {
        "author_placeOrder"
    }

    pub fn new(encoder: E) -> Self {
        RpcPlaceOrder {
            call_encoder: encoder,
        }
    }

    fn place_order(&self, _request: Request) -> RpcResult<(&str, bool, DirectRequestStatus)> {
        Ok(("ok", true, DirectRequestStatus::Ok))
    }
}

impl<E: RpcCallEncoder + Send + Sync + 'static> RpcMethodSync for RpcPlaceOrder<E> {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        E::call(params, &|r: Request| self.place_order(r))
    }
}

pub mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::rpc::rpc_call_encoder::tests::RpcCallEncoderMock;

    pub fn test_method_name_should_not_be_empty() {
        assert_eq!(
            RpcPlaceOrder::<RpcCallEncoderMock>::method_name().is_empty(),
            false
        );
    }

    pub fn test_given_incorrect_encoded_request_then_return_error() {
        // TODO construct a Params object to pass into the test
        // let rpc_place_order = RpcPlaceOrder {};
        //
        // rpc_place_order.call()
    }
}
