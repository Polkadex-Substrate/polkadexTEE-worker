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

use crate::polkadex_balance_storage::Balances;
use crate::polkadex_gateway::GatewayError;
use crate::rpc::polkadex_rpc_gateway::RpcGateway;
use polkadex_sgx_primitives::types::{Order, OrderUUID};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_types::{sgx_status_t, SgxResult};
use substratee_stf::TrustedCall;

/// Mock implementation to be used in unit testing
pub struct RpcGatewayMock {
    pub do_authorize: bool,
    pub balance_to_return: Option<Balances>,
    pub order_uuid_to_return: Option<OrderUUID>,
}

/// constructors
impl RpcGatewayMock {
    pub fn mock_balances(balances: Option<Balances>, do_authorize: bool) -> Self {
        RpcGatewayMock {
            do_authorize,
            balance_to_return: balances,
            order_uuid_to_return: None,
        }
    }

    pub fn mock_place_order(order_uuid: Option<OrderUUID>, do_authorize: bool) -> Self {
        RpcGatewayMock {
            do_authorize,
            balance_to_return: None,
            order_uuid_to_return: order_uuid,
        }
    }
}

impl RpcGateway for RpcGatewayMock {
    fn authorize_user(
        &self,
        _main_account: AccountId,
        _proxy_account: Option<AccountId>,
    ) -> Result<(), GatewayError> {
        match self.do_authorize {
            true => Ok(()),
            false => Err(GatewayError::ProxyNotRegisteredForMainAccount),
        }
    }

    fn authorize_trusted_call(&self, _trusted_call: &TrustedCall) -> Result<(), String> {
        match self.do_authorize {
            true => Ok(()),
            false => Err(String::from("Authorization failed")),
        }
    }

    fn get_balances(&self, _main_account: AccountId, _asset_it: AssetId) -> SgxResult<Balances> {
        match &self.balance_to_return {
            Some(b) => Ok(b.clone()),
            None => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }

    fn place_order(
        &self,
        _main_account: AccountId,
        _proxy_acc: Option<AccountId>,
        _order: Order,
    ) -> Result<OrderUUID, GatewayError> {
        match &self.order_uuid_to_return {
            Some(o) => Ok(o.clone()),
            None => Err(GatewayError::OrderNotFound),
        }
    }
}
