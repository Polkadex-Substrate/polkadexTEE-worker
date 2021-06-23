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

use crate::polkadex_gateway::GatewayError;
use codec::Encode;
use log::*;
use polkadex_sgx_primitives::{AccountId, AssetId, Balance};
use sgx_tstd::collections::HashMap;
use sgx_tstd::vec::Vec;

use crate::polkadex_balance_storage::balances::*;
use crate::polkadex_balance_storage::polkadex_balance_key::*;

pub type EncodedKey = Vec<u8>;

pub struct PolkadexBalanceStorage {
    /// map (tokenID, AccountID) -> (balance free, balance reserved)
    pub storage: HashMap<EncodedKey, Balances>,
}

impl PolkadexBalanceStorage {
    pub fn create() -> PolkadexBalanceStorage {
        PolkadexBalanceStorage {
            storage: HashMap::new(),
        }
    }

    pub fn read_balance(&self, token: AssetId, acc: AccountId) -> Option<&Balances> {
        let key = PolkadexBalanceKey::from(token, acc).encode();
        debug!("reading balance from key: {:?}", key);
        self.storage.get(&key)
    }

    pub fn initialize_balance(&mut self, token: AssetId, acc: AccountId, free: Balance) {
        let key = PolkadexBalanceKey::from(token, acc).encode();
        debug!("creating new entry for key: {:?}", key);
        self.storage.insert(key, Balances::from(free, 0u128));
    }

    pub fn set_free_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = amt;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn set_reserve_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.reserved = amt;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn deposit(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc.clone()).encode())
        {
            Some(balance) => {
                balance.free = balance.free.saturating_add(amt);
                Ok(())
            }
            None => {
                debug!("No entry available for given token- and AccountId, creating new.");
                self.initialize_balance(token, acc, amt);
                Ok(())
            }
        }
    }

    pub fn withdraw(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = balance.free.saturating_sub(amt);
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn reduce_free_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = balance
                    .free
                    .checked_sub(amt)
                    .ok_or(GatewayError::LimitOrderPriceNotFound)?; //FIXME Error type
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn increase_free_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc.clone()).encode())
        {
            Some(balance) => {
                balance.free = balance
                    .free
                    .checked_add(amt)
                    .ok_or(GatewayError::LimitOrderPriceNotFound)?; //FIXME Error Type
                Ok(())
            }
            None => {
                self.initialize_balance(token, acc, amt);
                Ok(())
            }
        }
    }
    // We can write functions which settle balances for two trades but we need to know the trade structure for it
}
