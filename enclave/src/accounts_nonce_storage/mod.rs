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

use chain_relay::{storage_proof::StorageProofChecker, Header};
use codec::Encode;
use frame_support::{metadata::StorageHasher, PalletId};
use log::*;
use polkadex_sgx_primitives::{AccountId, PolkadexAccount};
use sgx_types::{sgx_status_t, SgxResult};
use sp_runtime::traits::{AccountIdConversion, Header as HeaderT};
use sp_std::prelude::*;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

use crate::{
    // accounts_storage::{AccountsStorageError, PolkadexAccountsStorage},
    // nonce_storage::{NonceStorageError, PolkadexNonceStorage},
    polkadex_gateway::GatewayError,
    utils::UnwrapOrSgxErrorUnexpected,
};

pub mod accounts_storage;
pub mod nonce_storage;
pub use accounts_storage::*;
pub use nonce_storage::*;

static GLOBAL_ACCOUNTS_AND_NONCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn verify_pdex_account_read_proofs(
    header: Header,
    accounts: Vec<PolkadexAccount>,
) -> SgxResult<()> {
    let mut last_account: AccountId = PalletId(*b"polka/ga").into_account();
    for account in accounts.iter() {
        if account.account.prev == last_account {
            if let Some(actual) = StorageProofChecker::<<Header as HeaderT>::Hashing>::check_proof(
                header.state_root,
                &storage_map_key(
                    "OCEX",
                    "MainAccounts",
                    &account.account.current,
                    &StorageHasher::Blake2_128Concat,
                ),
                account.proof.to_vec(),
            )
            .sgx_error_with_log("Erroneous Storage Proof")?
            {
                if actual != account.account.encode() {
                    error!("Wrong storage value supplied");
                    return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                }
                if account.account.next.is_some() {
                    last_account = account.account.current.clone();
                } else {
                    break;
                }
            } else {
                error!("StorageProofChecker returned None");
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        } else {
            error!("Linkedlist is broken");
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
    }

    Ok(())
}

pub fn storage_map_key<K: Encode>(
    module_prefix: &str,
    storage_prefix: &str,
    mapkey1: &K,
    hasher1: &StorageHasher,
) -> Vec<u8> {
    let mut bytes = sp_core::twox_128(module_prefix.as_bytes()).to_vec();
    bytes.extend(&sp_core::twox_128(storage_prefix.as_bytes())[..]);
    bytes.extend(key_hash(mapkey1, hasher1));
    bytes
}

/// generates the key's hash depending on the StorageHasher selected
fn key_hash<K: Encode>(key: &K, hasher: &StorageHasher) -> Vec<u8> {
    let encoded_key = key.encode();
    match hasher {
        StorageHasher::Identity => encoded_key.to_vec(),
        StorageHasher::Blake2_128 => sp_core::blake2_128(&encoded_key).to_vec(),
        StorageHasher::Blake2_128Concat => {
            // copied from substrate Blake2_128Concat::hash since StorageHasher is not public
            let x: &[u8] = encoded_key.as_slice();
            sp_core::blake2_128(x)
                .iter()
                .chain(x.iter())
                .cloned()
                .collect::<Vec<_>>()
        }
        StorageHasher::Blake2_256 => sp_core::blake2_256(&encoded_key).to_vec(),
        StorageHasher::Twox128 => sp_core::twox_128(&encoded_key).to_vec(),
        StorageHasher::Twox256 => sp_core::twox_256(&encoded_key).to_vec(),
        StorageHasher::Twox64Concat => sp_core::twox_64(&encoded_key)
            .iter()
            .chain(&encoded_key)
            .cloned()
            .collect(),
    }
}

pub fn create_in_memory_accounts_and_nonce_storage(
    accounts: Vec<PolkadexAccount>,
) -> Result<(), GatewayError> {
    let storage = AccountsNonceStorage::create(accounts);
    let storage_ptr = Arc::new(SgxMutex::<AccountsNonceStorage>::new(storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ACCOUNTS_AND_NONCE_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct AccountsNonceStorage {
    pub accounts_storage: PolkadexAccountsStorage,
    pub nonce_storage: PolkadexNonceStorage,
}

impl AccountsNonceStorage {
    fn create(accounts: Vec<PolkadexAccount>) -> Self {
        Self {
            accounts_storage: PolkadexAccountsStorage::create(accounts.clone()),
            nonce_storage: PolkadexNonceStorage::create(accounts),
        }
    }

    fn register_main_account(&mut self, acc: AccountId) -> Result<(), AccountRegistryError> {
        self.accounts_storage.add_main_account(acc.clone())?;
        self.nonce_storage.initialize_nonce(acc);
        Ok(())
    }

    fn register_proxy_account(
        &mut self,
        acc: AccountId,
        proxy: AccountId,
    ) -> Result<(), AccountRegistryError> {
        self.accounts_storage.add_proxy(acc, proxy.clone())?;
        self.nonce_storage.initialize_nonce(proxy);
        Ok(())
    }

    fn remove_main_account(&mut self, acc: AccountId) -> Result<(), AccountRegistryError> {
        self.accounts_storage.remove_main_account(acc.clone())?;
        self.nonce_storage.remove_nonce(acc);
        Ok(())
    }

    fn remove_proxy_account(
        &mut self,
        acc: AccountId,
        proxy: AccountId,
    ) -> Result<(), AccountRegistryError> {
        self.accounts_storage.remove_proxy(acc, proxy.clone())?;
        self.nonce_storage.remove_nonce(proxy);
        Ok(())
    }

    fn check_if_main_account_registered(&self, acc: AccountId) -> bool {
        self.accounts_storage.accounts.contains_key(&acc.encode())
    }

    fn check_if_proxy_registered(
        &self,
        acc: AccountId,
        proxy: AccountId,
    ) -> Result<bool, AccountRegistryError> {
        if let Some(list_of_proxies) = self.accounts_storage.accounts.get(&acc.encode()) {
            Ok(list_of_proxies.contains(&proxy))
        } else {
            Err(AccountRegistryError::MainAccountNoRegistedForGivenProxy)
        }
    }

    // Nonce related functions

    fn validate_and_increment_nonce(
        &mut self,
        acc: AccountId,
        nonce: u32,
    ) -> Result<(), AccountRegistryError> {
        if self.nonce_storage.read_nonce(acc.clone())? != nonce {
            return Err(AccountRegistryError::CouldNotLoadRegistry); //FIX
        }
        self.nonce_storage.increment_nonce(acc)?;
        Ok(())
    }
}

pub fn check_if_main_account_registered(acc: AccountId) -> Result<bool, AccountRegistryError> {
    // Acquire lock on proxy_registry
    let mutex = load_registry()?;
    let storage: SgxMutexGuard<AccountsNonceStorage> = mutex
        .lock()
        .map_err(|_| AccountRegistryError::CouldNotGetMutex)?;
    let result = storage.check_if_main_account_registered(acc);
    Ok(result)
}

pub fn check_if_proxy_registered(
    acc: AccountId,
    proxy: AccountId,
) -> Result<bool, AccountRegistryError> {
    // Acquire lock on proxy_registry
    let mutex = load_registry()?;
    let storage: SgxMutexGuard<AccountsNonceStorage> = mutex
        .lock()
        .map_err(|_| AccountRegistryError::CouldNotGetMutex)?;

    storage.check_if_proxy_registered(acc, proxy)
}

pub fn add_main_account(main_acc: AccountId) -> Result<(), AccountRegistryError> {
    // Aquire lock on proxy_registry
    let mutex = load_registry()?;
    let mut storage: SgxMutexGuard<AccountsNonceStorage> = mutex
        .lock()
        .map_err(|_| AccountRegistryError::CouldNotGetMutex)?;
    storage.register_main_account(main_acc)?;
    Ok(())
}

pub fn remove_main_account(main_acc: AccountId) -> Result<(), AccountRegistryError> {
    // Aquire lock on proxy_registry
    let mutex = load_registry()?;
    let mut storage: SgxMutexGuard<AccountsNonceStorage> = mutex
        .lock()
        .map_err(|_| AccountRegistryError::CouldNotGetMutex)?;
    storage.remove_main_account(main_acc)?;
    Ok(())
}

pub fn add_proxy(main_acc: AccountId, proxy: AccountId) -> Result<(), AccountRegistryError> {
    // Aquire lock on proxy_registry
    let mutex = load_registry()?;
    let mut storage: SgxMutexGuard<AccountsNonceStorage> = mutex
        .lock()
        .map_err(|_| AccountRegistryError::CouldNotGetMutex)?;
    storage.register_proxy_account(main_acc, proxy)?;
    Ok(())
}

pub fn remove_proxy(main_acc: AccountId, proxy: AccountId) -> Result<(), AccountRegistryError> {
    // Aquire lock on proxy_registry
    let mutex = load_registry()?;
    let mut storage: SgxMutexGuard<AccountsNonceStorage> = mutex
        .lock()
        .map_err(|_| AccountRegistryError::CouldNotGetMutex)?;
    storage.remove_proxy_account(main_acc, proxy)?;
    Ok(())
}

pub fn load_registry() -> Result<&'static SgxMutex<AccountsNonceStorage>, AccountRegistryError> {
    let ptr = GLOBAL_ACCOUNTS_AND_NONCE_STORAGE.load(Ordering::SeqCst)
        as *mut SgxMutex<AccountsNonceStorage>;
    if ptr.is_null() {
        error!("Null pointer to polkadex account registry");
        Err(AccountRegistryError::CouldNotLoadRegistry)
    } else {
        Ok(unsafe { &*ptr })
    }
}

//Nonce related functions

pub fn auth_user_validate_increment_nonce(
    acc: AccountId,
    proxy_acc: Option<AccountId>,
    nonce: u32,
) -> Result<(), AccountRegistryError> {
    let mutex = load_registry()?;
    let mut storage: SgxMutexGuard<AccountsNonceStorage> = mutex
        .lock()
        .map_err(|_| AccountRegistryError::CouldNotGetMutex)?;

    match proxy_acc {
        Some(proxy) => {
            if !storage.check_if_proxy_registered(acc.clone(), proxy)? {
                return Err(AccountRegistryError::ProxyAccountNoRegistedForGivenMainAccount);
            }
        }
        None => {
            if !storage.check_if_main_account_registered(acc.clone()) {
                return Err(AccountRegistryError::MainAccountNoRegistedForGivenProxy);
            }
        }
    }
    if storage.nonce_storage.read_nonce(acc.clone())? != nonce {
        return Err(AccountRegistryError::NonceValidationFailed);
    }
    storage.nonce_storage.increment_nonce(acc)?;
    Ok(())
}

#[derive(Eq, Debug, PartialEq, PartialOrd)]
pub enum AccountRegistryError {
    /// Could not load the registry for some reason
    CouldNotLoadRegistry,
    /// Could not get mutex
    CouldNotGetMutex,
    /// No registed main account for given proxy
    MainAccountNoRegistedForGivenProxy,
    /// No registed proxy account for given main account
    ProxyAccountNoRegistedForGivenMainAccount,
    /// Nonce validation failed (didn't match)
    NonceValidationFailed,
    /// PolkadexAccountsStorage Error
    AccountStorageError(AccountsStorageError),
    /// PolkadexNonceStorage Error
    NonceStorageError(NonceStorageError),
}

impl From<AccountsStorageError> for AccountRegistryError {
    fn from(error: AccountsStorageError) -> AccountRegistryError {
        AccountRegistryError::AccountStorageError(error)
    }
}

impl From<NonceStorageError> for AccountRegistryError {
    fn from(error: NonceStorageError) -> AccountRegistryError {
        AccountRegistryError::NonceStorageError(error)
    }
}

pub mod tests {
    use super::{create_in_memory_accounts_and_nonce_storage, load_registry, AccountsNonceStorage};
    use codec::Encode;
    use polkadex_sgx_primitives::{AccountId, LinkedAccount, PolkadexAccount};
    use sp_core::{ed25519 as ed25519_core, Pair};

    pub fn create_and_load_registry() {
        assert!(create_in_memory_accounts_and_nonce_storage(vec![]).is_ok());
        assert_eq!(
            *load_registry().unwrap().lock().unwrap(),
            AccountsNonceStorage::create(vec![])
        );
    }

    pub fn create_accounts_nonce_storage() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let storage_empty: AccountsNonceStorage = AccountsNonceStorage::create(vec![]);
        let storage_with_account: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);
        assert!(!storage_empty
            .accounts_storage
            .accounts
            .contains_key(&account_id.encode()));
        assert!(!storage_empty
            .nonce_storage
            .storage
            .contains_key(&account_id.encode()));

        assert_eq!(
            storage_with_account
                .accounts_storage
                .accounts
                .get(&account_id.encode()),
            Some(&vec![])
        );
        assert_eq!(
            storage_with_account
                .nonce_storage
                .storage
                .get(&account_id.encode()),
            Some(&0u32)
        );
    }

    pub fn register_main_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let mut storage: AccountsNonceStorage = AccountsNonceStorage::create(vec![]);
        assert!(storage.register_main_account(account_id.clone()).is_ok());
        assert!(storage
            .accounts_storage
            .accounts
            .contains_key(&account_id.encode()));
        assert_eq!(storage.nonce_storage.read_nonce(account_id), Ok(0u32));
    }

    pub fn remove_main_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let mut storage: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);

        assert!(storage.remove_main_account(account_id.clone()).is_ok());
        assert!(!storage
            .accounts_storage
            .accounts
            .contains_key(&account_id.encode()));
        assert!(storage.nonce_storage.read_nonce(account_id).is_err());
    }

    pub fn register_proxy_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();
        let mut storage: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);

        assert!(storage
            .register_proxy_account(account_id.clone(), proxy_id.clone())
            .is_ok());
        assert_eq!(
            storage.accounts_storage.accounts.get(&account_id.encode()),
            Some(&vec![proxy_id.clone()])
        );
        assert_eq!(storage.nonce_storage.read_nonce(proxy_id), Ok(0u32));
    }

    pub fn remove_proxy_account() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();
        let mut storage: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![proxy_id.clone()],
                },
                proof: vec![],
            }]);

        assert!(storage
            .remove_proxy_account(account_id.clone(), proxy_id.clone())
            .is_ok());
        assert_eq!(
            storage.accounts_storage.accounts.get(&account_id.encode()),
            Some(&vec![])
        );
        assert!(storage.nonce_storage.read_nonce(proxy_id).is_err());
    }

    pub fn check_if_main_account_registered() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let storage_with_account: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);
        let storage_empty: AccountsNonceStorage = AccountsNonceStorage::create(vec![]);
        assert!(storage_with_account.check_if_main_account_registered(account_id.clone()));
        assert!(!storage_empty.check_if_main_account_registered(account_id));
    }

    pub fn check_if_proxy_registered() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let proxy_id: AccountId =
            ed25519_core::Pair::from_seed(b"23456789012345678901234567890123")
                .public()
                .into();
        let storage_with_proxy: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![proxy_id.clone()],
                },
                proof: vec![],
            }]);
        let storage_with_account: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);
        let storage_empty: AccountsNonceStorage = AccountsNonceStorage::create(vec![]);
        assert_eq!(
            storage_with_proxy.check_if_proxy_registered(account_id.clone(), proxy_id.clone()),
            Ok(true)
        );
        assert_eq!(
            storage_with_account.check_if_proxy_registered(account_id.clone(), proxy_id.clone()),
            Ok(false)
        );
        assert!(storage_empty
            .check_if_proxy_registered(account_id, proxy_id)
            .is_err());
    }

    pub fn validate_and_increment_nonce() {
        let account_id: AccountId =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
                .public()
                .into();
        let mut storage: AccountsNonceStorage =
            AccountsNonceStorage::create(vec![PolkadexAccount {
                account: LinkedAccount {
                    prev: account_id.clone(),
                    current: account_id.clone(),
                    next: None,
                    proxies: vec![],
                },
                proof: vec![],
            }]);

        // Validate nonce and increment while checking if it did correctly, all 3 should be ok
        assert!(storage
            .validate_and_increment_nonce(account_id.clone(), 0u32)
            .is_ok());
        assert!(storage
            .validate_and_increment_nonce(account_id.clone(), 1u32)
            .is_ok());
        assert!(storage
            .validate_and_increment_nonce(account_id.clone(), 2u32)
            .is_ok());

        // Try to validate the wrong nonce, should be error
        assert!(storage
            .validate_and_increment_nonce(account_id.clone(), 0u32)
            .is_err());

        // Check to see if the previous error didn't wrongfully increment the nonce, should be ok
        assert!(storage
            .validate_and_increment_nonce(account_id, 3u32)
            .is_ok());
    }
}
