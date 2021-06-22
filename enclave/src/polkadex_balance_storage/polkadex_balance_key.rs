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
use codec::{Decode, Encode};
use polkadex_sgx_primitives::{AccountId, AssetId};


#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PolkadexBalanceKey {
    pub asset_id: AssetId,
    pub account_id: AccountId,
}

impl PolkadexBalanceKey {
    pub fn from(asset_id: AssetId, account_id: AccountId) -> Self {
        Self {
            asset_id,
            account_id,
        }
    }
}