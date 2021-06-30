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

use crate::polkadex_balance_storage::{
    lock_storage_and_get_balances, lock_storage_and_withdraw, Balances,
};

use crate::polkadex_nonce_storage::{lock_storage_and_get_nonce, lock_storage_and_increment_nonce, NonceHandler};

use crate::execute_ocex_release_extrinsic;
use crate::openfinex::openfinex_api_impl::OpenFinexApiImpl;
use crate::openfinex::openfinex_client::OpenFinexClientInterface;
use crate::polkadex_gateway::{authenticate_user, GatewayError, OpenfinexPolkaDexGateway};
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

    /// verifies that the proxy account (if any) is authorized to represent the main account
    /// given a trusted call inside a trusted operation (convenience function)
    fn authorize_trusted_call(
        &self,
        trusted_operation: TrustedOperation,
    ) -> Result<TrustedCall, String>;

    /// get the balance of a certain asset ID for a given account
    fn get_balances(&self, main_account: AccountId, asset_it: AssetId) -> SgxResult<Balances>;

    /// get the nonce for a given account
    fn get_nonce(&self, main_account: AccountId) -> SgxResult<NonceHandler>;

    /// increment the nonce for a given account
    fn increment_nonce(&self, main_account: AccountId) -> SgxResult<()>;

    /// place an order
    fn place_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: Order,
    ) -> Result<(), GatewayError>;

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

    fn authorize_trusted_call(
        &self,
        trusted_operation: TrustedOperation,
    ) -> Result<TrustedCall, String> {
        let trusted_call = match trusted_operation {
            TrustedOperation::direct_call(tcs) => Ok(tcs.call),
            _ => {
                error!("Trusted calls entering via RPC must be direct");
                Err(RpcCallStatus::operation_type_mismatch.to_string())
            }
        }?;

        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account().clone();

        match self.authorize_user(main_account.clone(), proxy_account.clone()) {
            Ok(()) => Ok(trusted_call),
            Err(e) => {
                error!("Could not find account within registry");
                Err(format!("Authorization error: {}", e))
            }
        }
    }

    fn get_balances(&self, main_account: AccountId, asset_id: AssetId) -> SgxResult<Balances> {
        match lock_storage_and_get_balances(main_account, asset_id) {
            Ok(balance) => Ok(balance),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }

    fn get_nonce(&self, main_account: AccountId) -> SgxResult<NonceHandler> {
        match lock_storage_and_get_nonce(main_account.clone()) {
            Ok(nonce) => Ok(nonce),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }

    fn increment_nonce(&self, main_account: AccountId) -> SgxResult<()> {
        lock_storage_and_increment_nonce(main_account.clone())
    }

    fn place_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: Order,
    ) -> Result<(), GatewayError> {
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
            Ok(_) => execute_ocex_release_extrinsic(main_account.clone(), token, amount),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }
}
