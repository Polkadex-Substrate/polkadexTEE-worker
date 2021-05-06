use chain_relay::{Header, storage_proof::StorageProofChecker};
use sp_runtime::ModuleId;
use polkadex_primitives::PolkadexAccount;
use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use sp_runtime::traits::{Header as HeaderT, AccountIdConversion};
use sp_std::prelude::*;
use substratee_stf::{
    AccountId, Getter, ShardIdentifier, Stf, TrustedCall, TrustedCallSigned, TrustedGetterSigned,
};
//use std::collections::HashMap;
// TODO: Fix this import
use std::sync::{Arc, atomic::{AtomicPtr, Ordering}, SgxMutex};
use sgx_tstd::collections::HashMap;
use sgx_tstd::hash::Hash;
use crate::utils::UnwrapOrSgxErrorUnexpected;
use core::hash::Hasher;
use core::ops::Deref;
use multibase::Base;
use codec::Encode;


static GLOBAL_ACCOUNTS_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn verify_pdex_account_read_proofs(
    header: Header,
    accounts: Vec<PolkadexAccount>) -> SgxResult<()> {
    let mut last_account: AccountId = ModuleId(*b"polka/ga").into_account();
    for account in accounts.iter() {
        if account.account.prev == last_account {
            StorageProofChecker::<<Header as HeaderT>::Hashing>::check_proof(
                header.state_root,
                account.account.current.as_ref(), // QUESTION: How is this key defined? What about storage prefix?
                account.proof.to_vec(),
            )
                .sgx_error_with_log("Erroneous StorageProof")?;

            last_account = account.account.next.clone().unwrap();
        }
    };

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
    accounts: HashMap<Vec<u8>, Vec<AccountId>>
}

impl PolkadexAccountsStorage {
    pub fn create(accounts: Vec<PolkadexAccount>) -> PolkadexAccountsStorage {
        let mut in_memory_map: PolkadexAccountsStorage = PolkadexAccountsStorage {
            accounts: HashMap::new(),
        };
        for account in accounts {
            in_memory_map.accounts.insert(account.account.current.encode(), account.account.proxies);
        }
        in_memory_map
    }

    pub fn check_main_account(acc: AccountId) -> SgxResult<bool> {
        let polkadex_map = Self::load_storage()?;
        Ok(polkadex_map.accounts.contains_key(&*acc.encode()))
    }

    pub fn check_proxy_account(main_acc: AccountId, proxy: AccountId) -> SgxResult<bool> {
        let polkadex_map = Self::load_storage()?;
        let list_of_proxies = polkadex_map.accounts.get(&*main_acc.encode()).unwrap(); //FIXME: Remove Unwrap
        Ok(list_of_proxies.contains(&proxy))
    }

    pub fn load_storage() -> SgxResult<Arc<PolkadexAccountsStorage>>{
        let ptr = GLOBAL_ACCOUNTS_STORAGE.load(Ordering::SeqCst)
            as *mut SgxMutex<PolkadexAccountsStorage>;
        if ptr.is_null() { return Err(sgx_status_t::SGX_ERROR_UNEXPECTED) };
        let ptr: &SgxMutex<PolkadexAccountsStorage> = unsafe{&* ptr};
        let pool_guard = ptr.lock().unwrap();
        let registry: Arc<PolkadexAccountsStorage> = Arc::new(*pool_guard);
        Ok(registry)
    }

    //pub fn insert_storage() -> SgxResult<Arc<&PolkadexAccountsStorage>>{


    //pub fn inser_proxy
}




