
use chain_relay::{storage_proof::StorageProofChecker, Header};
use codec::Encode;
use core::hash::Hasher;
use core::ops::Deref;
use multibase::Base;
use polkadex_primitives::PolkadexAccount;
use sgx_tstd::collections::HashMap;
use sgx_tstd::hash::Hash;
use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use sp_runtime::traits::{AccountIdConversion, Header as HeaderT};
use sp_runtime::ModuleId;
use sp_std::prelude::*;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};
use substratee_stf::{
    AccountId, Getter, ShardIdentifier, Stf, TrustedCall, TrustedCallSigned, TrustedGetterSigned,
};

//use std::collections::HashMap;
// TODO: Fix this import
use crate::utils::UnwrapOrSgxErrorUnexpected;

static GLOBAL_ACCOUNTS_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn verify_pdex_account_read_proofs(
    header: Header,
    accounts: Vec<PolkadexAccount>,
) -> SgxResult<()> {
    let mut last_account: AccountId = ModuleId(*b"polka/ga").into_account();
    for account in accounts.iter() {
        if account.account.prev == last_account.as_ref() {
            StorageProofChecker::<<Header as HeaderT>::Hashing>::check_proof(
                header.state_root,
                account.account.current.as_ref(), // QUESTION: How is this key defined? What about storage prefix?
                account.proof.to_vec(),
            )
            .sgx_error_with_log("Erroneous StorageProof")?;

            last_account = AccountId::from(account.account.next.clone().unwrap());
        }
    }

    Ok(())
}

pub fn create_in_memory_account_storage(accounts: Vec<PolkadexAccount>) -> SgxResult<()> {
    let accounts_storage = PolkadexAccountsStorage::create(accounts);
    let storage_ptr = Arc::new(SgxMutex::<PolkadexAccountsStorage>::new(accounts_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ACCOUNTS_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

/// Access that pointer
pub struct PolkadexAccountsStorage {
    pub(crate) accounts: HashMap<[u8;32], Vec<[u8;32]>>,
}

impl PolkadexAccountsStorage {

    pub fn from_hashmap(hashmap: HashMap<[u8;32], Vec<[u8;32]>>) -> Self{
        Self{
            accounts: hashmap
        }
    }

    pub fn create(accounts: Vec<PolkadexAccount>) -> PolkadexAccountsStorage {
        let mut in_memory_map: PolkadexAccountsStorage = PolkadexAccountsStorage {
            accounts: HashMap::new(),
        };
        for account in accounts {
            in_memory_map
                .accounts
                .insert(account.account.current, account.account.proxies);
        }
        in_memory_map
    }

    pub fn add_main_account(&mut self, acc: [u8;32]) {
        let vec: Vec<[u8;32]> = Vec::new();
        self.accounts.insert(acc, vec);
    }

    pub fn remove_main_account(&mut self, acc: [u8;32]) {
        self.accounts.remove(&acc);
    }

    pub fn add_proxy(&mut self, main: [u8;32], proxy: [u8;32]) {
        let proxies: &mut Vec<[u8;32]> = self.accounts.get_mut(&main).unwrap();
        proxies.push(proxy);
    }

    pub fn remove_proxy(&mut self, main: [u8;32], proxy: [u8;32]) {
        let proxies: &mut Vec<[u8;32]> = self.accounts.get_mut(&main).unwrap();
        for index in 0..proxies.len() {
            if proxies.get(index).unwrap() == &proxy {
                proxies.remove(index);
                break;
            }
        }
    }
}

pub fn check_main_account(acc: [u8;32]) -> SgxResult<bool> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.accounts.contains_key(&acc))
}

pub fn check_proxy_account(main_acc: [u8;32], proxy: [u8;32]) -> SgxResult<bool> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();

    let list_of_proxies = proxy_storage.accounts.get(&main_acc).unwrap(); //FIXME: Remove Unwrap
    Ok(list_of_proxies.contains(&proxy))
}

pub fn add_main_account(main_acc: [u8;32]) -> SgxResult<()> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.add_main_account(main_acc))
}

pub fn remove_main_account(main_acc: [u8;32]) -> SgxResult<()> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.remove_main_account(main_acc))
}

pub fn add_proxy(main_acc: [u8;32], proxy: [u8;32]) -> SgxResult<()> {
    // Aquire lock on proxy_registry
    let mutex = load_proxy_registry()?;
    let mut proxy_storage: SgxMutexGuard<PolkadexAccountsStorage> = mutex.lock().unwrap();
    Ok(proxy_storage.add_proxy(main_acc, proxy))
}

pub fn remove_proxy(main_acc: [u8;32], proxy: [u8;32]) -> SgxResult<()> {
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
