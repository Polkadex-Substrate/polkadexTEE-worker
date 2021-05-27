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

use crate::polkadex_balance_storage::{lock_storage_and_get_balances, Balances};
use crate::polkadex_gateway::{authenticate_user, GatewayError};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_types::SgxResult;

/// Gateway trait from RPC API -> Polkadex gateway implementation
pub trait RpcGateway: Send + Sync {
    fn authorize_user(
        &self,
        main_account: AccountId,
        proxy_account: Option<AccountId>,
    ) -> Result<(), GatewayError>;

    fn get_balances(&self, main_account: AccountId, asset_it: AssetId) -> SgxResult<Balances>;
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

    fn get_balances(&self, main_account: AccountId, asset_id: AssetId) -> SgxResult<Balances> {
        lock_storage_and_get_balances(main_account, asset_id)
    }
}
