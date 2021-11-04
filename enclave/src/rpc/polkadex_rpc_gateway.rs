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
use alloc::{string::String, string::ToString};
use log::error;

use crate::channel_storage::{load_sender, ChannelType};
use crate::execute_ocex_release_extrinsic;
use crate::openfinex::openfinex_api_impl::OpenFinexApiImpl;
use crate::openfinex::openfinex_client::OpenFinexClientInterface;
use crate::polkadex_balance_storage::{
    lock_storage_and_get_balances, lock_storage_and_withdraw, Balances,
};
use crate::polkadex_cache::cache_api::RequestId;
use crate::polkadex_gateway::{
    authenticate_user, authenticate_user_and_validate_nonce, register_account, GatewayError,
    OpenfinexPolkaDexGateway,
};
use crate::rpc::rpc_info::RpcCallStatus;
use polkadex_sgx_primitives::types::{CancelOrder, Order};
use polkadex_sgx_primitives::{AccountId, AssetId, Balance};
use sgx_types::{sgx_status_t, SgxResult};
use substratee_stf::{TrustedCall, TrustedOperation};

/// Gateway trait from RPC API -> Polkadex gateway implementation
pub trait RpcGateway: Send + Sync {
    /// verifies that the proxy account (if any) is authorized to represent the main account
    fn authorize_user(
        &self,
        main_account: AccountId,
        proxy_account: Option<AccountId>,
    ) -> Result<(), GatewayError>;

    /// Register account.
    fn register_account(
        &self,
        main_account: AccountId,
        proxy_account: AccountId,
    ) -> Result<(), GatewayError>;

    /// verifies that the proxy account (if any) is authorized to represent the main account and also verifies if the provided nonce matches the one in the storage
    fn authorize_user_nonce(
        &self,
        main_account: AccountId,
        proxy_account: Option<AccountId>,
        nonce: u32,
    ) -> Result<(), GatewayError>;

    /// verifies that the proxy account (if any) is authorized to represent the main account
    /// given a trusted call inside a trusted operation (convenience function)
    fn authorize_trusted_call(
        &self,
        trusted_operation: TrustedOperation,
    ) -> Result<TrustedCall, String>;

    /// get the balance of a certain asset ID for a given account
    fn get_balances(&self, main_account: AccountId, asset_it: AssetId) -> SgxResult<Balances>;

    /// get the nonce for a given account
    fn nonce(&self, main_account: AccountId) -> SgxResult<u32>;

    /// place an order
    fn place_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: Order,
    ) -> Result<RequestId, GatewayError>;

    /// cancel an order, identified by UUID
    fn cancel_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        cancel_order: CancelOrder,
    ) -> Result<(), GatewayError>;

    /// withdraw funds from main account
    fn withdraw(&self, main_account: AccountId, token: AssetId, amount: Balance) -> SgxResult<()>;
}

pub struct PolkadexRpcGateway {}

impl RpcGateway for PolkadexRpcGateway {
    fn authorize_user(
        &self,
        main_account: AccountId,
        proxy_account: Option<AccountId>,
    ) -> Result<(), GatewayError> {
        authenticate_user(main_account, proxy_account)
    }

    fn register_account(
        &self,
        main_account: AccountId,
        proxy_account: AccountId,
    ) -> Result<(), GatewayError> {
        register_account(main_account, proxy_account)
    }

    fn authorize_user_nonce(
        &self,
        main_account: AccountId,
        proxy_account: Option<AccountId>,
        nonce: u32,
    ) -> Result<(), GatewayError> {
        let result =
            authenticate_user_and_validate_nonce(main_account.clone(), proxy_account, nonce);
        if result.is_ok() {
            load_sender()
                .map_err(|_| GatewayError::UnableToLoadPointer)?
                .send(ChannelType::Nonce(main_account, nonce + 1))
                .map_err(|_| GatewayError::UndefinedBehaviour)?;
        }
        result
    }

    #[cfg(not(feature = "benchmarking"))]
    fn authorize_trusted_call(
        &self,
        trusted_operation: TrustedOperation,
    ) -> Result<TrustedCall, String> {
        let (trusted_call, nonce) = match trusted_operation {
            TrustedOperation::direct_call(tcs) => Ok((tcs.call, tcs.nonce)),
            _ => {
                error!("Trusted calls entering via RPC must be direct");
                Err(RpcCallStatus::operation_type_mismatch.to_string())
            }
        }?;

        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account();

        match self.authorize_user_nonce(main_account, proxy_account, nonce) {
            Ok(()) => Ok(trusted_call),
            Err(e) => {
                error!("Could not find account within registry: {:?}", e);
                Err(format!("Authorization error: {}", e))
            }
        }
    }

    #[cfg(feature = "benchmarking")]
    fn authorize_trusted_call(
        &self,
        trusted_operation: TrustedOperation,
    ) -> Result<TrustedCall, String> {
        let (trusted_call, nonce) = match trusted_operation {
            TrustedOperation::direct_call(tcs) => Ok((tcs.call, tcs.nonce)),
            _ => {
                error!("Trusted calls entering via RPC must be direct");
                Err(RpcCallStatus::operation_type_mismatch.to_string())
            }
        }?;

        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account();

        Ok(trusted_call)
    }

    fn get_balances(&self, main_account: AccountId, asset_id: AssetId) -> SgxResult<Balances> {
        match lock_storage_and_get_balances(main_account, asset_id) {
            Ok(balance) => Ok(balance),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }

    fn nonce(&self, main_account: AccountId) -> SgxResult<u32> {
        match crate::accounts_nonce_storage::get_nonce(main_account) {
            Ok(nonce) => Ok(nonce),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }

    fn place_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: Order,
    ) -> Result<RequestId, GatewayError> {
        let gateway = OpenfinexPolkaDexGateway::new(OpenFinexApiImpl::new(
            OpenFinexClientInterface::new(0), // FIXME: for now hardcoded 0, but we should change that to..?
        ));
        gateway.place_order(main_account, proxy_acc, order)
    }

    fn cancel_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: CancelOrder,
    ) -> Result<(), GatewayError> {
        let gateway = OpenfinexPolkaDexGateway::new(OpenFinexApiImpl::new(
            OpenFinexClientInterface::new(0), // FIXME: for now hardcoded 0, but we should change that to..?
        ));
        gateway.cancel_order(main_account, proxy_acc, order)
    }

    fn withdraw(&self, main_account: AccountId, token: AssetId, amount: Balance) -> SgxResult<()> {
        match lock_storage_and_withdraw(main_account.clone(), token, amount) {
            Ok(_) => execute_ocex_release_extrinsic(main_account, token, amount),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }
}

pub mod tests {
    use crate::rpc::mocks::dummy_builder::create_dummy_account;
    use crate::rpc::mocks::dummy_builder::create_dummy_request;
    use crate::rpc::mocks::dummy_builder::sign_trusted_call;
    use crate::rpc::mocks::rpc_gateway_mock::RpcGatewayMock;
    use crate::rpc::mocks::trusted_operation_extractor_mock::TrustedOperationExtractorMock;
    use crate::rpc::polkadex_rpc_gateway::TrustedOperation;
    use crate::rpc::rpc_withdraw::RpcWithdraw;
    use crate::TrustedCall;
    use polkadex_sgx_primitives::{AccountId, AssetId};
    use sgx_tstd::boxed::Box;
    use sp_application_crypto::Pair;

    pub fn test_rejecting_outdated_nonce() {
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation(0u32)),
        });

        let top_extractor1 = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation(0u32)),
        });

        let mut rpc_gateway = Box::new(RpcGatewayMock::mock_withdraw(true));

        let rpc_withdraw = RpcWithdraw::new(top_extractor, rpc_gateway.clone());
        assert_eq!(0u32, rpc_gateway.nonce);
        rpc_gateway.increment_nonce();
        let rpc_withdraw1 = RpcWithdraw::new(top_extractor1, rpc_gateway.clone());
        assert_eq!(1u32, rpc_gateway.nonce);

        rpc_withdraw.method_impl(create_dummy_request()).unwrap();

        let result = rpc_withdraw1.method_impl(create_dummy_request());

        assert!(result.is_err());
    }

    pub fn test_successful_call_with_nonce() {
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation(0u32)),
        });

        let top_extractor1 = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_withdraw_order_operation(1u32)),
        });

        let mut rpc_gateway = Box::new(RpcGatewayMock::mock_withdraw(true));

        let rpc_withdraw = RpcWithdraw::new(top_extractor, rpc_gateway.clone());
        assert_eq!(0u32, rpc_gateway.nonce);
        rpc_gateway.increment_nonce();
        let rpc_withdraw1 = RpcWithdraw::new(top_extractor1, rpc_gateway.clone());
        assert_eq!(1u32, rpc_gateway.nonce);

        rpc_withdraw.method_impl(create_dummy_request()).unwrap();

        let result = rpc_withdraw1.method_impl(create_dummy_request());

        assert!(result.is_ok());
    }

    fn create_withdraw_order_operation(nonce: u32) -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let trusted_call = TrustedCall::withdraw(account_id, AssetId::DOT, 1000, None);

        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair, nonce);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
