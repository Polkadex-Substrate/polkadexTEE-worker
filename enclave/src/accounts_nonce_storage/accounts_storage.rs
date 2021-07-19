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

use codec::Encode;
use log::*;
use polkadex_sgx_primitives::{AccountId, PolkadexAccount};
use sgx_tstd::collections::HashMap;
use sgx_tstd::vec::Vec;

pub type EncodedAccountId = Vec<u8>;

#[derive(Debug, PartialEq)]
pub struct PolkadexAccountsStorage {
    /// map AccountId -> Vec<AccountId>
    pub accounts: HashMap<EncodedAccountId, Vec<AccountId>>,
}

impl PolkadexAccountsStorage {
    #[allow(unused)]
    pub fn from_hashmap(hashmap: HashMap<EncodedAccountId, Vec<AccountId>>) -> Self {
        Self { accounts: hashmap }
    }

    pub fn create(accounts: Vec<PolkadexAccount>) -> PolkadexAccountsStorage {
        let mut in_memory_map: PolkadexAccountsStorage = PolkadexAccountsStorage {
            accounts: HashMap::new(),
        };
        for account in accounts {
            in_memory_map
                .accounts
                .insert(account.account.current.encode(), account.account.proxies);
        }
        in_memory_map
    }

    pub fn add_main_account(&mut self, acc: AccountId) -> Result<(), AccountsStorageError> {
        if self.accounts.contains_key(&acc.encode()) {
            warn!("Given account is registered");
            return Err(AccountsStorageError::AccountAlreadyRegistered);
        };
        let vec: Vec<AccountId> = Vec::new();
        self.accounts.insert(acc.encode(), vec);
        Ok(())
    }

    pub fn remove_main_account(&mut self, acc: AccountId) -> Result<(), AccountsStorageError> {
        if !self.accounts.contains_key(&acc.encode()) {
            warn!("Given account is not registered");
            return Err(AccountsStorageError::AccountNotRegistered);
        };
        self.accounts.remove(&acc.encode());
        Ok(())
    }

    pub fn add_proxy(
        &mut self,
        main: AccountId,
        proxy: AccountId,
    ) -> Result<(), AccountsStorageError> {
        if let Some(proxies) = self.accounts.get_mut(&main.encode()) {
            if !proxies.contains(&proxy) {
                proxies.push(proxy);
                return Ok(());
            }
            warn!("Given Proxy is already registered");
            return Err(AccountsStorageError::ProxyAlreadyRegistered);
        };
        warn!("Given Account is not registered");
        Err(AccountsStorageError::AccountNotRegistered)
    }

    pub fn remove_proxy(
        &mut self,
        main: AccountId,
        proxy: AccountId,
    ) -> Result<(), AccountsStorageError> {
        if let Some(proxies) = self.accounts.get_mut(&main.encode()) {
            if proxies.contains(&proxy) {
                let index = proxies.iter().position(|x| *x == proxy).unwrap();
                proxies.remove(index);
                return Ok(());
            }
            warn!("Given Proxy is not registered");
            return Err(AccountsStorageError::ProxyNotRegistered);
        };
        warn!("Given Account is not registered");
        Err(AccountsStorageError::AccountNotRegistered)
    }
}

#[derive(Eq, Debug, PartialEq, PartialOrd)]
pub enum AccountsStorageError {
    /// The account is already registered
    AccountAlreadyRegistered,
    /// The account is not registered
    AccountNotRegistered,
    /// The proxy is already registered
    ProxyAlreadyRegistered,
    /// The proxy is not registered
    ProxyNotRegistered,
}

pub mod tests {
    use super::{EncodedAccountId, PolkadexAccountsStorage};
    use codec::Encode;
    use polkadex_sgx_primitives::{AccountId, LinkedAccount, PolkadexAccount};
    use sgx_tstd::collections::HashMap;
    use sgx_tstd::vec::Vec;
    use sp_core::{ed25519 as ed25519_core, Pair};

    pub fn create_accounts_storage_from_hashmap() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();

        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();

        let second_account_id: AccountId =
            ed25519_core::Pair::from_seed(b"34567890123456789012345678901234")
                .public()
                .into();

        let mut hashmap: HashMap<EncodedAccountId, Vec<AccountId>> = HashMap::new();
        hashmap.insert(account_id.encode(), vec![proxy_id]);
        hashmap.insert(second_account_id.encode(), vec![]);

        assert_eq!(
            PolkadexAccountsStorage::from_hashmap(hashmap.clone()),
            PolkadexAccountsStorage { accounts: hashmap }
        );
    }

    pub fn create_accounts_storage() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let storage_empty: PolkadexAccountsStorage = PolkadexAccountsStorage::create(vec![]);
        let storage_with_account: PolkadexAccountsStorage =
            PolkadexAccountsStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);
        assert!(!storage_empty.accounts.contains_key(&account_id.encode()));
        assert_eq!(
            storage_with_account.accounts.get(&account_id.encode()),
            Some(&vec![])
        );
    }

    pub fn adding_main_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let mut accounts_storage: PolkadexAccountsStorage = PolkadexAccountsStorage::create(vec![]);
        assert!(accounts_storage
            .add_main_account(account_id.clone())
            .is_ok());
        assert!(accounts_storage.accounts.contains_key(&account_id.encode()));
    }

    pub fn removing_main_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let mut accounts_storage: PolkadexAccountsStorage =
            PolkadexAccountsStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);
        assert!(accounts_storage
            .remove_main_account(account_id.clone())
            .is_ok());

        assert!(!accounts_storage.accounts.contains_key(&account_id.encode()));
    }

    pub fn adding_proxy_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();
        let mut accounts_storage: PolkadexAccountsStorage =
            PolkadexAccountsStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);
        assert!(accounts_storage
            .add_proxy(account_id.clone(), proxy_id.clone())
            .is_ok());

        assert_eq!(
            accounts_storage.accounts.get(&account_id.encode()),
            Some(&vec![proxy_id])
        );
    }

    pub fn removing_proxy_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();
        let mut accounts_storage: PolkadexAccountsStorage =
            PolkadexAccountsStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![proxy_id.clone()],
                },
                proof: vec![],
            }]);
        assert!(accounts_storage
            .remove_proxy(account_id.clone(), proxy_id)
            .is_ok());

        assert_eq!(
            accounts_storage.accounts.get(&account_id.encode()),
            Some(&vec![])
        );
    }

    pub fn adding_already_registered_accounts() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();
        let mut accounts_storage: PolkadexAccountsStorage =
            PolkadexAccountsStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![proxy_id.clone()],
                },
                proof: vec![],
            }]);
        assert!(accounts_storage.accounts.contains_key(&account_id.encode()));

        assert!(accounts_storage
            .add_main_account(account_id.clone())
            .is_err());
        assert!(accounts_storage.add_proxy(account_id, proxy_id).is_err());
    }

    pub fn removing_not_registered_accounts() {
        let registered_account_id: AccountId =
            ed25519_core::Pair::from_seed(b"34567890123456789012345678901234")
                .public()
                .into();
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();

        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();
        let mut accounts_storage: PolkadexAccountsStorage =
            PolkadexAccountsStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: registered_account_id.clone(),
                    current: registered_account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);
        assert!(!accounts_storage.accounts.contains_key(&account_id.encode()));
        assert!(accounts_storage
            .remove_main_account(account_id.clone())
            .is_err());
        assert!(accounts_storage
            .remove_proxy(account_id, proxy_id.clone())
            .is_err());
        assert!(accounts_storage
            .remove_proxy(registered_account_id, proxy_id)
            .is_err());
    }
}
