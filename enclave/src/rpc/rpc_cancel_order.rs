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

use crate::rpc::polkadex_rpc_gateway::RpcGateway;
use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::RpcCallStatus;
use crate::rpc::trusted_operation_verifier::TrustedOperationExtractor;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use polkadex_sgx_primitives::types::OrderUUID;
use substratee_stf::{TrustedCall, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcCancelOrder {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcCancelOrder {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcCancelOrder {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> Result<((), bool, DirectRequestStatus), String> {
        debug!("entering cancel_order RPC");

        let verified_trusted_operation =
            self.top_extractor.get_verified_trusted_operation(request)?;

        let trusted_call = self
            .rpc_gateway
            .authorize_trusted_call(verified_trusted_operation)?;

        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account();

        let order_id = match trusted_call {
            TrustedCall::cancel_order(_, order, _) => Ok(order),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        match self
            .rpc_gateway
            .cancel_order(main_account, proxy_account.clone(), order_id)
        {
            Ok(()) => Ok(((), false, DirectRequestStatus::Ok)),
            Err(e) => Err(String::from(e.to_string())),
        }
    }
}

impl RpcCall for RpcCancelOrder {
    fn name() -> String {
        "cancel_order".to_string()
    }
}

impl RpcMethodSync for RpcCancelOrder {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}

pub mod tests {

    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_request, sign_trusted_call, create_dummy_cancel_order,
    };
    use crate::rpc::mocks::rpc_gateway_mock::RpcGatewayMock;
    use crate::rpc::mocks::trusted_operation_extractor_mock::TrustedOperationExtractorMock;
    use codec::Encode;
    use sp_core::Pair;
    use substratee_stf::AccountId;

    pub fn test_given_valid_order_id_return_success() {
        let order_id = "lojoif93j2lngfa".encode();

        let rpc_gateway = Box::new(RpcGatewayMock::mock_cancel_order(
            Some(order_id.clone()),
            true,
        ));

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_cancel_order_operation(order_id)),
        });

        let request = create_dummy_request();

        let rpc_cancel_order = RpcCancelOrder::new(top_extractor, rpc_gateway);

        let result = rpc_cancel_order.method_impl(request).unwrap();

        assert_eq!(result.2, DirectRequestStatus::Ok);
    }

    pub fn test_given_order_id_mismatch_then_fail() {
        let order_id = "lojoif93j2lngfa".encode();

        let rpc_gateway = Box::new(RpcGatewayMock::mock_cancel_order(
            Some("other_id_that_doesnt_match".encode()),
            true,
        ));

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_cancel_order_operation(order_id)),
        });

        let request = create_dummy_request();

        let rpc_cancel_order = RpcCancelOrder::new(top_extractor, rpc_gateway);

        let result = rpc_cancel_order.method_impl(request);

        assert!(result.is_err());
    }

    fn create_cancel_order_operation(order_id: OrderUUID) -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();
        let cancel_order = create_dummy_cancel_order(account_id.clone(), order_id);

        let trusted_call = TrustedCall::cancel_order(account_id.clone(), cancel_order, None);
        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair, 0u32);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
