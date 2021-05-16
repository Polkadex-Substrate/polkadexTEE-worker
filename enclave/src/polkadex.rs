use chain_relay::{Header, storage_proof::StorageProofChecker};
use codec::Encode;
use core::hash::Hasher;
use core::ops::Deref;
use frame_support::{metadata::StorageHasher, PalletId};
use log::*;
use multibase::Base;
use polkadex_sgx_primitives::{AccountId, PolkadexAccount};
use sgx_tstd::collections::HashMap;
use sgx_tstd::hash::Hash;
use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use sp_core::blake2_256;
use sp_runtime::traits::{AccountIdConversion, Header as HeaderT};
use sp_std::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicPtr, Ordering}, SgxMutex, SgxMutexGuard,
};

//use std::collections::HashMap;
// TODO: Fix this import
use crate::utils::UnwrapOrSgxErrorUnexpected;

static GLOBAL_ACCOUNTS_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

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
                if &actual != &account.account.encode() {
                    error!("Wrong storage value supplied");
                    return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                }
                last_account = account.account.next.clone().unwrap();
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
        StorageHasher::Twox64Concat => sp_core::twox_64(&encoded_key).to_vec(),
    }
}

pub fn create_in_memory_account_storage(accounts: Vec<PolkadexAccount>) -> SgxResult<()> {
    let accounts_storage = PolkadexAccountsStorage::create(accounts);
    let storage_ptr = Arc::new(SgxMutex::<PolkadexAccountsStorage>::new(accounts_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ACCOUNTS_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub type EncodedAccountId = Vec<u8>;

/// Access that pointer
pub struct PolkadexAccountsStorage {
    pub(crate) accounts: HashMap<EncodedAccountId, Vec<AccountId>>,
}

impl PolkadexAccountsStorage {
    pub fn from_hashmap(hashmap: HashMap<EncodedAccountId, Vec<AccountId>>) -> Self {
        Self { accounts: hashmap }
    }

    pub fn create(accounts: Vec<PolkadexAccount>) -> PolkadexAccountsStorage {
        let mut in_memory_map: PolkadexAccountsStorage = PolkadexAccountsStorage {
            accounts: HashMap::new(),
        };
        for account in accounts {
            in_memory_map.accounts.insert(
                account.account.current.encode(),
                account.account.proxies,
            );
        }
        in_memory_map
    }

    pub fn add_main_account(&mut self, acc: AccountId) {
        if self.accounts.contains_key(&acc.encode()) {
            warn!("Given account is registered");
            return;
        };
        let vec: Vec<AccountId> = Vec::new();
        self.accounts.insert(acc.encode(), vec);
    }

    pub fn remove_main_account(&mut self, acc: AccountId) {
        if !self.accounts.contains_key(&acc.encode()) {
            warn!("Given account is not registered");
            return;
        };
        self.accounts.remove(&acc.encode());
    }

    pub fn add_proxy(&mut self, main: AccountId, proxy: AccountId) {
        if let Some(proxies) = self.accounts.get_mut(&main.encode()) {
            if !proxies.contains(&proxy) {
                proxies.push(proxy);
                return;
            }
            warn!("Given Proxy is already registered");
        };
        warn!("Given Account is not registered");
    }

    pub fn remove_proxy(&mut self, main: AccountId, proxy: AccountId) {
        if let Some(proxies) = self.accounts.get_mut(&main.encode()) {
            if proxies.contains(&proxy) {
                let index = proxies.iter().position(|x| *x == proxy).unwrap();
                proxies.remove(index);
                return;
            }
            warn!("Given Proxy is not registered");
        };
        warn!("Given Account is not registered");
    }
}

pub fn check_if_main_account_registered(acc: AccountId) -> SgxResult<bool> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage
        .accounts
        .contains_key(&acc.encode()))
}

pub fn check_if_proxy_registered(main_acc: AccountId, proxy: AccountId) -> SgxResult<bool> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();

    if let Some(list_of_proxies) = proxy_storage
        .accounts
        .get(&main_acc.encode())
    {
        Ok(list_of_proxies.contains(&proxy))
    } else {
        warn!("Main account not present");
        Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
    }
}

pub fn add_main_account(main_acc: AccountId) -> SgxResult<()> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.add_main_account(main_acc))
}

pub fn remove_main_account(main_acc: AccountId) -> SgxResult<()> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.remove_main_account(main_acc))
}

pub fn add_proxy(main_acc: AccountId, proxy: AccountId) -> SgxResult<()> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.add_proxy(main_acc, proxy))
}

pub fn remove_proxy(main_acc: AccountId, proxy: AccountId) -> SgxResult<()> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.remove_proxy(main_acc, proxy))
}

pub fn load_proxy_registry() -> SgxResult<&'static SgxMutex<PolkadexAccountsStorage>> {
    let ptr =
        GLOBAL_ACCOUNTS_STORAGE.load(Ordering::SeqCst) as *mut SgxMutex<PolkadexAccountsStorage>;
    if ptr.is_null() {
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}
