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
use codec::Encode;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use polkadex_sgx_primitives::types::{CancelOrder, DirectRequest};
use sp_application_crypto::Pair;
use substratee_stf::TrustedCall;
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcEditOrder {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcEditOrder {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcEditOrder {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(
        &self,
        request: DirectRequest,
        test: bool,
    ) -> Result<(RequestId, bool, DirectRequestStatus), String> {
        debug!("entering edit_order RPC");

        let verified_trusted_operation =
            self.top_extractor.get_verified_trusted_operation(request)?;

        let trusted_call = self
            .rpc_gateway
            .authorize_trusted_call(verified_trusted_operation)?;

        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account();

        let edit_order = match trusted_call {
            TrustedCall::edit_order(_, edit_order, _) => Ok(edit_order),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let edited_order = if test {
            if edit_order.order_id.clone() != "lojoif93j2lngfa".encode() {
                return Err(String::from("Failed to retrieve order from orderbook"));
            } else {
                crate::rpc::mocks::dummy_builder::create_dummy_order(
                    crate::rpc::mocks::dummy_builder::create_dummy_account()
                        .public()
                        .into(),
                )
                .edit(edit_order.value)
            }
        } else {
            crate::polkadex_orderbook_storage::lock_storage_and_read_order(
                edit_order.order_id.clone(),
            )
            .map_err(|_| String::from("Failed to retrieve order from orderbook"))?
            .edit(edit_order.value)
        };

        self.rpc_gateway
            .cancel_order(
                main_account.clone(),
                proxy_account.clone(),
                CancelOrder::from_order(edited_order.clone(), edit_order.order_id),
            )
            .map_err(|_| String::from("Failed to cancel order"))?;

        let result = match self
            .rpc_gateway
            .place_order(main_account, proxy_account, edited_order)
        {
            Ok(request_id) => Ok(request_id),
            Err(e) => Err(e.to_string()),
        }?;

        Ok((result, true, DirectRequestStatus::Ok))
    }
}

impl RpcCall for RpcEditOrder {
    fn name() -> String {
        "edit_order".to_string()
    }
}

impl RpcMethodSync for RpcEditOrder {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r, false))
    }
}

pub mod tests {

    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_edit_order, create_dummy_order, create_dummy_request,
        sign_trusted_call,
    };
    use crate::rpc::mocks::rpc_gateway_mock::RpcGatewayMock;
    use crate::rpc::mocks::trusted_operation_extractor_mock::TrustedOperationExtractorMock;
    use codec::Encode;
    use polkadex_sgx_primitives::types::OrderUUID;
    use polkadex_sgx_primitives::AccountId;
    use sp_core::Pair;
    use substratee_stf::TrustedOperation;

    pub fn test_given_valid_order_id_return_success() {
        let order_id = "lojoif93j2lngfa".encode();

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_edit_order_operation(order_id.clone())),
        });

        let rpc_gateway = Box::new(RpcGatewayMock::mock_edit_order(
            Some(order_id),
            Some(create_dummy_order(create_dummy_account().public().into())),
            true,
        ));

        let request = create_dummy_request();

        let rpc_edit_order = RpcEditOrder::new(top_extractor, rpc_gateway);

        let result = rpc_edit_order.method_impl(request, true).unwrap();

        assert_eq!(result.2, DirectRequestStatus::Ok);
    }

    pub fn test_given_order_id_mismatch_then_fail() {
        let order_id = "lojoif93j2lngfa".encode();

        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_edit_order_operation(order_id)),
        });

        let rpc_gateway = Box::new(RpcGatewayMock::mock_edit_order(
            Some("other_id_that_doesnt_match".encode()),
            Some(create_dummy_order(create_dummy_account().public().into())),
            true,
        ));

        let request = create_dummy_request();

        let rpc_edit_order = RpcEditOrder::new(top_extractor, rpc_gateway);

        let result = rpc_edit_order.method_impl(request, true);

        assert!(result.is_err());
    }

    fn create_edit_order_operation(order_id: OrderUUID) -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();
        let edit_order = create_dummy_edit_order(order_id);

        let trusted_call = TrustedCall::edit_order(account_id, edit_order, None);
        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair, 0u32);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
