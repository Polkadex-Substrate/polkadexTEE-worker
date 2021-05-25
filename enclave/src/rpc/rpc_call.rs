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

use alloc::{str, string::String};

use crate::rpc::rpc_call_encoder::RpcCallEncoder;
use crate::rpc::rpc_info::RpcInfo;
use core::marker::{Send, Sync};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::*;
use polkadex_sgx_primitives::types::DirectRequest;
use substratee_worker_primitives::DirectRequestStatus;

pub type RpcMethodImpl =
    dyn Fn(DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)> + Sync;

/// RPC call structure
pub struct RpcCall<E, F>
where
    E: RpcCallEncoder + Send + Sync + 'static,
    F: Fn(DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)>
        + Sync
        + ?Sized
        + 'static,
{
    method_name: &'static str,
    method_impl: &'static F,
    call_encoder: E,
}

impl<E, F> RpcCall<E, F>
where
    E: RpcCallEncoder + Send + Sync + 'static,
    F: Fn(DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)>
        + Sync
        + ?Sized
        + 'static,
{
    pub fn method_name(&self) -> &'static str {
        self.method_name
    }

    // FIXME: this produces a warning, because we're not using the call encoder as field,
    // but merely as associated function in the implementation. However, if we don't have a field,
    // the compiler gives an error that type parameter 'E' is not used, even though it clearly is
    pub fn new(name: &'static str, method: &'static F, encoder: E) -> Self {
        RpcCall {
            method_name: name,
            method_impl: method,
            call_encoder: encoder,
        }
    }
}

impl<E, F> RpcMethodSync for RpcCall<E, F>
where
    E: RpcCallEncoder + Send + Sync + 'static,
    F: Fn(DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)>
        + Sync
        + ?Sized
        + 'static,
{
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        E::call(params, &|r: DirectRequest| (self.method_impl)(r))
    }
}

pub mod tests {

    use super::*;
    use crate::rpc::rpc_call_encoder::tests::RpcCallEncoderMock;
    use crate::rpc::rpc_info::RpcCallStatus;
    use jsonrpc_core::futures::executor::block_on;

    pub fn test_method_name_should_not_be_empty() {
        let rpc_call = create_test_rpc_call();

        assert_eq!(rpc_call.method_name().is_empty(), false);
    }

    pub fn test_given_none_params_return_ok_result() {
        let rpc_call = create_test_rpc_call();

        let result = block_on(rpc_call.call(Params::None));
        let result_value = result.unwrap();

        assert!(!result_value.is_null());
    }

    fn create_test_rpc_call() -> RpcCall<RpcCallEncoderMock, RpcMethodImpl> {
        RpcCall::new(
            "test_call",
            &|_r: DirectRequest| {
                Ok((
                    RpcInfo::from(RpcCallStatus::operation_success),
                    false,
                    DirectRequestStatus::Ok,
                ))
            },
            RpcCallEncoderMock {},
        )
    }
}
