// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

pub extern crate alloc;
use alloc::{boxed::Box, string::String, string::ToString};

use crate::polkadex_cache::cache_api::RequestId;
use crate::rpc::polkadex_rpc_gateway::RpcGateway;
use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::RpcCallStatus;
use crate::rpc::trusted_operation_verifier::TrustedOperationExtractor;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use polkadex_sgx_primitives::types::{DirectRequest, OrderUUID};
use substratee_stf::{TrustedCall, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcPlaceOrder {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcPlaceOrder {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcPlaceOrder {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> Result<(RequestId, bool, DirectRequestStatus), String> {
        debug!("entering place_order RPC");

        let verified_trusted_operation =
            self.top_extractor.get_verified_trusted_operation(request)?;

        let trusted_call = self
            .rpc_gateway
            .authorize_trusted_call(verified_trusted_operation)?;

        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account();

        let order = match trusted_call {
            TrustedCall::place_order(_, order, _) => Ok(order),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let result = match self
            .rpc_gateway
            .place_order(main_account, proxy_account, order)
        {
            Ok(request_id) => Ok(request_id), //FIXME: this is not ok!
            Err(e) => Err(e.to_string()),
        }?;

        Ok((result, true, DirectRequestStatus::Ok))
    }
}

impl RpcCall for RpcPlaceOrder {
    fn name() -> String {
        "place_order".to_string()
    }
}

impl RpcMethodSync for RpcPlaceOrder {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}

pub mod tests {

    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_order, create_dummy_request, sign_trusted_call,
    };
    use crate::rpc::mocks::rpc_gateway_mock::RpcGatewayMock;
    use crate::rpc::mocks::trusted_operation_extractor_mock::TrustedOperationExtractorMock;
    use codec::Encode;
    use polkadex_sgx_primitives::AccountId;
    use sp_core::Pair;

    pub fn test_given_valid_call_return_order_uuid() {
        let order_uuid = "lkas903jfaj3".encode();

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_place_order_operation()),
        });

        let rpc_gateway = Box::new(RpcGatewayMock::mock_place_order(Some(order_uuid), true));

        let request = create_dummy_request();

        let rpc_place_order = RpcPlaceOrder::new(top_extractor, rpc_gateway);

        let result = rpc_place_order.method_impl(request).unwrap();

        //assert_eq!(result.0, order_uuid); // TODO - does not return the order UUID anymore (non-blocking call)
        assert_eq!(result.2, DirectRequestStatus::Ok);
    }

    fn create_place_order_operation() -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();
        let order = create_dummy_order(account_id.clone());

        let trusted_call = TrustedCall::place_order(account_id, order, None);

        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair, 0u32);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
