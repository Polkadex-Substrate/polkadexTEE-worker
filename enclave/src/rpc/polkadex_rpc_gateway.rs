// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü.
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
use alloc::string::String;

use crate::polkadex_balance_storage::{lock_storage_and_get_balances, Balances};
use crate::polkadex_gateway::{authenticate_user, place_order, GatewayError};
use polkadex_sgx_primitives::types::{Order, OrderUUID};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_types::SgxResult;
use substratee_stf::TrustedCall;

/// Gateway trait from RPC API -> Polkadex gateway implementation
pub trait RpcGateway: Send + Sync {
    /// verifies that the proxy account (if any) is authorized to represent the main account
    fn authorize_user(
        &self,
        main_account: AccountId,
        proxy_account: Option<AccountId>,
    ) -> Result<(), GatewayError>;

    /// verifies that the proxy account (if any) is authorized to represent the main account
    /// given a trusted call (convenience function)
    fn authorize_trusted_call(&self, trusted_call: &TrustedCall) -> Result<(), String>;

    /// get the balance of a certain asset ID for a given account
    fn get_balances(&self, main_account: AccountId, asset_it: AssetId) -> SgxResult<Balances>;

    /// place an order
    fn place_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: Order,
    ) -> Result<OrderUUID, GatewayError>;
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

    fn authorize_trusted_call(&self, trusted_call: &TrustedCall) -> Result<(), String> {
        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account().clone();

        match self.authorize_user(main_account.clone(), proxy_account.clone()) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Authorization error: {}", e)),
        }
    }

    fn get_balances(&self, main_account: AccountId, asset_id: AssetId) -> SgxResult<Balances> {
        lock_storage_and_get_balances(main_account, asset_id)
    }

    fn place_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: Order,
    ) -> Result<OrderUUID, GatewayError> {
        place_order(main_account, proxy_acc, order)
    }
}
