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
use alloc::string::String;

use crate::polkadex_balance_storage::Balances;
use crate::polkadex_gateway::GatewayError;
use crate::rpc::polkadex_rpc_gateway::RpcGateway;
use log::error;
use polkadex_sgx_primitives::types::{CancelOrder, Order, OrderUUID};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_types::{sgx_status_t, SgxResult};
use substratee_stf::{TrustedCall, TrustedOperation};

#[derive(Clone)]
/// Mock implementation to be used in unit testing
pub struct RpcGatewayMock {
    pub do_authorize: bool,
    pub balance_to_return: Option<Balances>,
    pub order_uuid: Option<OrderUUID>,
    pub nonce: u32,
}

/// constructors
impl RpcGatewayMock {
    fn default() -> Self {
        RpcGatewayMock {
            do_authorize: false,
            balance_to_return: None,
            order_uuid: None,
            nonce: 0u32,
        }
    }

    pub fn mock_balances(balances: Option<Balances>, do_authorize: bool) -> Self {
        let mut get_balances_mock = RpcGatewayMock::default();
        get_balances_mock.balance_to_return = balances;
        get_balances_mock.do_authorize = do_authorize;
        get_balances_mock
    }

    pub fn mock_place_order(order_uuid: Option<OrderUUID>, do_authorize: bool) -> Self {
        let mut get_place_order_mock = RpcGatewayMock::default();
        get_place_order_mock.order_uuid = order_uuid;
        get_place_order_mock.do_authorize = do_authorize;
        get_place_order_mock
    }

    pub fn mock_cancel_order(order_uuid: Option<OrderUUID>, do_authorize: bool) -> Self {
        let mut get_place_order_mock = RpcGatewayMock::default();
        get_place_order_mock.order_uuid = order_uuid;
        get_place_order_mock.do_authorize = do_authorize;
        get_place_order_mock
    }

    pub fn mock_withdraw(do_authorize: bool) -> Self {
        let mut withdraw_mock = RpcGatewayMock::default();
        withdraw_mock.do_authorize = do_authorize;
        withdraw_mock
    }

    pub fn increment_nonce(&mut self) {
        self.nonce += 1u32;
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

    fn authorize_user_nonce(
        &self,
        _main_account: AccountId,
        _proxy_account: Option<AccountId>,
        _nonce: u32,
    ) -> Result<(), GatewayError> {
        match self.do_authorize {
            true => Ok(()),
            false => Err(GatewayError::ProxyNotRegisteredForMainAccount),
        }
    }

    fn authorize_trusted_call(
        &self,
        trusted_operation: TrustedOperation,
    ) -> Result<TrustedCall, String> {
        let (call, nonce) = match self.do_authorize {
            true => match trusted_operation {
                TrustedOperation::direct_call(tcs) => Ok((tcs.call, tcs.nonce)),
                _ => Err(String::from("Trusted operation is not a direct call")),
            },
            false => Err(String::from("Authorization failed")),
        }?;

        crate::channel_storage::load_sender()
            .unwrap()
            .send(crate::channel_storage::ChannelType::Nonce(
                call.main_account().clone(),
                nonce + 1,
            ))
            .unwrap();
        error!("SENT NONCE");

        if self.nonce == nonce {
            Ok(call)
        } else {
            Err(String::from("failed cause nonce doesn't match"))
        }
    }

    fn get_balances(&self, _main_account: AccountId, _asset_it: AssetId) -> SgxResult<Balances> {
        match &self.balance_to_return {
            Some(b) => Ok(b.clone()),
            None => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }

    fn nonce(&self, _main_account: AccountId) -> SgxResult<u32> {
        Ok(self.nonce)
    }

    fn place_order(
        &self,
        _main_account: AccountId,
        _proxy_acc: Option<AccountId>,
        _order: Order,
    ) -> Result<(), GatewayError> {
        match &self.order_uuid {
            Some(_) => Ok(()),
            None => Err(GatewayError::OrderNotFound),
        }
    }

    fn cancel_order(
        &self,
        _main_account: AccountId,
        _proxy_acc: Option<AccountId>,
        cancel_order: CancelOrder,
    ) -> Result<(), GatewayError> {
        match &self.order_uuid {
            Some(o) => {
                if o.eq(&cancel_order.order_id) {
                    Ok(())
                } else {
                    Err(GatewayError::OrderNotFound)
                }
            }
            None => Err(GatewayError::OrderNotFound),
        }
    }

    fn withdraw(&self, _main_account: AccountId, _token: AssetId, _amount: u128) -> SgxResult<()> {
        Ok(())
    }
}
