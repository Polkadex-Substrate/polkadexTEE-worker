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
use codec::{Decode, Encode};
use log::*;
use polkadex_sgx_primitives::{
    AccountId, AssetId, Balance, Balances, BalancesData, PolkadexBalanceKey,
};
use sgx_tstd::collections::HashMap;
use sgx_tstd::vec::Vec;

use crate::channel_storage::{load_sender, ChannelType};

pub type EncodedKey = Vec<u8>;

#[derive(Debug)]
pub struct PolkadexBalanceStorage {
    /// map (tokenID, AccountID) -> (balance free, balance reserved)
    pub storage: HashMap<EncodedKey, polkadex_sgx_primitives::Balances>,
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

    pub fn initialize_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        free: Balance,
    ) -> Result<(), GatewayError> {
        let key = PolkadexBalanceKey::from(token, acc).encode();
        debug!("creating new entry for key: {:?}", key);
        self.storage.insert(key, Balances::from(free, 0u128));
        self.balance_change()?;
        Ok(())
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
                self.balance_change()?;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                Err(GatewayError::AccountIdOrAssetIdNotFound)
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
                self.balance_change()?;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                Err(GatewayError::AccountIdOrAssetIdNotFound)
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
                self.balance_change()?;
                Ok(())
            }
            None => {
                debug!("No entry available for given token- and AccountId, creating new.");
                self.initialize_balance(token, acc, amt)?;
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
                self.balance_change()?;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                Err(GatewayError::AccountIdOrAssetIdNotFound)
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
                self.balance_change()?;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                Err(GatewayError::AccountIdOrAssetIdNotFound)
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
                self.balance_change()?;
                Ok(())
            }
            None => {
                self.initialize_balance(token, acc, amt)?;
                Ok(())
            }
        }
    }

    fn balance_change(&mut self) -> Result<(), GatewayError> {
        load_sender()
            .map_err(|_| GatewayError::UnableToLoadPointer)?
            .send(ChannelType::Balances(self.prepare_to_export()))
            .map_err(|_| GatewayError::UndefinedBehaviour)?;
        Ok(())
    }

    pub fn extend_from_disk_data(&mut self, data: Vec<BalancesData>) {
        self.storage.extend(
            data.into_iter()
                .map(|entry| (entry.account.encode(), entry.balances)),
        );
    }

    pub fn prepare_to_export(&mut self) -> Vec<BalancesData> {
        self.storage
            .iter()
            .map(|(account, balances)| {
                let account = PolkadexBalanceKey::decode(&mut account.as_slice()).unwrap();
                BalancesData {
                    account,
                    balances: *balances,
                }
            })
            .collect::<Vec<BalancesData>>()
    }

    // We can write functions which settle balances for two trades but we need to know the trade structure for it
}
