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
use substratee_stf::{TrustedCall, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

use crate::polkadex_nonce_storage::lock_storage_and_get_nonce;

pub struct RpcWithdraw {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcWithdraw {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcWithdraw {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> Result<((), bool, DirectRequestStatus), String> {
        debug!("entering withdraw RPC");

        let verified_trusted_operation =
            self.top_extractor.get_verified_trusted_operation(request)?;

        let trusted_call = self
            .rpc_gateway
            .authorize_trusted_call(verified_trusted_operation)?;

        let main_account = trusted_call.main_account().clone();

        let asset_with_amount = match trusted_call {
            TrustedCall::withdraw(_, asset_id, amount, _) => Ok((asset_id, amount)),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let asset_id = asset_with_amount.0;
        let amount = asset_with_amount.1;

        match self.rpc_gateway.withdraw(main_account, asset_id, amount) {
            Ok(()) => Ok(((), false, DirectRequestStatus::Ok)),
            Err(e) => Err(format!("Failed to withdraw: {}", e.as_str())),
        }
    }
}

impl RpcCall for RpcWithdraw {
    fn name() -> String {
        "withdraw".to_string()
    }
}

impl RpcMethodSync for RpcWithdraw {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}

pub mod tests {

    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_request, sign_trusted_call,
    };
    use crate::rpc::mocks::rpc_gateway_mock::RpcGatewayMock;
    use crate::rpc::mocks::trusted_operation_extractor_mock::TrustedOperationExtractorMock;
    use polkadex_sgx_primitives::{AccountId, AssetId};
    use sp_core::Pair;

    pub fn test_given_valid_call_then_succeed() {
        let rpc_gateway = Box::new(RpcGatewayMock::mock_withdraw(true));

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation()),
        });

        let rpc_withdraw = RpcWithdraw::new(top_extractor, rpc_gateway);

        let result = rpc_withdraw.method_impl(create_dummy_request());

        assert!(result.is_ok());
    }

    pub fn test_incrementing_nonce_wont_match() {
        let mut rpc_gateway = Box::new(RpcGatewayMock::mock_withdraw(true));

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation()),
        });

        let top_extractor1 = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation()),
        });

        let rpc_withdraw = RpcWithdraw::new(top_extractor, rpc_gateway.clone());
        assert_eq!(0u32, rpc_gateway.nonce);
        rpc_gateway.increment_nonce();
        let rpc_withdraw1 = RpcWithdraw::new(top_extractor1, rpc_gateway.clone());
        assert_eq!(1u32, rpc_gateway.nonce);


        rpc_withdraw.method_impl(create_dummy_request()).unwrap();



        let result = rpc_withdraw1.method_impl(create_dummy_request());

        assert!(result.is_err());
    }

    pub fn test_given_unauthorized_access_then_return_error() {
        let rpc_gateway = Box::new(RpcGatewayMock::mock_withdraw(false));

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation()),
        });

        let rpc_withdraw = RpcWithdraw::new(top_extractor, rpc_gateway);

        let result = rpc_withdraw.method_impl(create_dummy_request());

        assert!(result.is_err());
    }

    fn create_withdraw_order_operation() -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let trusted_call = TrustedCall::withdraw(account_id.clone(), AssetId::DOT, 1000, None);

        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair, 0u32);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
