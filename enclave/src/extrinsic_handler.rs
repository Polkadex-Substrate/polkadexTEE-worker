use log::*;
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_tstd::collections::hash_map::HashMap;
use sgx_tstd::collections::vec_deque::VecDeque;
use sgx_tstd::vec::Vec;
use sgx_types::{sgx_status_t, SgxResult};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

static GLOBAL_POLKADEX_EXTRINSIC_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct PendingExtrinsicHelper {
    pub active_set: VecDeque<(u32, Vec<u8>)>,
    pub finalized_nonce: u32,
    pub unfinalized_nonce: u32,
}

impl PendingExtrinsicHelper {
    pub fn create() -> Self {
        Self {
            active_set: VecDeque::new(),
            finalized_nonce: 0u32,
            unfinalized_nonce: 0u32,
        }
    }

    pub fn add_active_set_element(&mut self, nonce: u32, element: Vec<u8>) {
        self.active_set.push_front((nonce, element));
    }

    pub fn get_unfinalized_nonce(&self) -> u32 {
        self.unfinalized_nonce
    }

    pub fn get_finalized_nonce(&self) -> u32 {
        self.finalized_nonce
    }

    pub fn increase_unfinalized_nonce(&mut self) {
        self.unfinalized_nonce += 1;
    }

    pub fn update_finalzied_nonce(&mut self, new_nonce: u32) {
        self.finalized_nonce = new_nonce;
    }

    pub fn update_unfinalzied_nonce(&mut self, new_nonce: u32) {
        self.unfinalized_nonce = new_nonce;
    }
}

pub fn create_in_memory_extrinsic_storage() -> SgxResult<()> {
    let extrinsic_storage = PendingExtrinsicHelper::create();
    let storage_ptr = Arc::new(SgxMutex::<PendingExtrinsicHelper>::new(extrinsic_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_POLKADEX_EXTRINSIC_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub fn load_extrinsic_storage() -> SgxResult<&'static SgxMutex<PendingExtrinsicHelper>> {
    let ptr = GLOBAL_POLKADEX_EXTRINSIC_STORAGE.load(Ordering::SeqCst)
        as *mut SgxMutex<PendingExtrinsicHelper>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}

pub fn lock_and_update_nonce(nonce: u32) -> SgxResult<()> {
    let mutex = load_extrinsic_storage()?;
    let mut extrinsic_storage: SgxMutexGuard<PendingExtrinsicHelper> = mutex.lock().unwrap();
    extrinsic_storage.clone().update_finalzied_nonce(nonce);
    Ok(())
}

pub fn lock_and_update_active_set() -> SgxResult<()> {
    let mutex = load_extrinsic_storage()?;
    let mut extrinsic_storage: SgxMutexGuard<PendingExtrinsicHelper> = mutex.lock().unwrap();
    extrinsic_storage.active_set = extrinsic_storage
        .active_set
        .into_iter()
        .filter(|&item| item.0 < extrinsic_storage.clone().finalized_nonce)
        .collect();
    if extrinsic_storage.active_set.get(0)

    Ok(())
}

//Change to SgxResult
pub fn lock_and_get_unconfirmed_transaction() -> SgxResult<Option<Vec<u8>>>{
    let mutex = load_extrinsic_storage()?;
    let mut extrinsic_storage: SgxMutexGuard<PendingExtrinsicHelper> = mutex.lock().unwrap();
    if extrinsic_storage.active_set.get(0).cloned().unwrap().0 > extrinsic_storage.get_finalized_nonce() {
        let unconfirmed_transaction = extrinsic_storage.active_set.get(0).cloned().unwrap();
        Ok(Some(unconfirmed_transaction.1))
    } else{
        Ok(None)
    }
}

pub fn lock_and_is_active_set_not_empty() -> SgxResult<bool>{
    let mutex = load_extrinsic_storage()?;
    let mut extrinsic_storage: SgxMutexGuard<PendingExtrinsicHelper> = mutex.lock().unwrap();
    Ok(extrinsic_storage.active_set.len() != 0)
}

pub fn lock_and_get_unfinalized_nonce() -> SgxResult<u32> {
    let mutex = load_extrinsic_storage()?;
    let mut extrinsic_storage: SgxMutexGuard<PendingExtrinsicHelper> = mutex.lock().unwrap();
    Ok(extrinsic_storage.get_unfinalized_nonce())
}

pub fn lock_and_increase_unfinalized_nonce() -> SgxResult<()> {
    let mutex = load_extrinsic_storage()?;
    let mut extrinsic_storage: SgxMutexGuard<PendingExtrinsicHelper> = mutex.lock().unwrap();
    extrinsic_storage.increase_unfinalized_nonce();
    Ok(())
}

pub fn failed_transaction_handler() {
    let mutex = load_extrinsic_storage()?;
    let mut extrinsic_storage: SgxMutexGuard<PendingExtrinsicHelper> = mutex.lock().unwrap();
    extrinsic_storage.increase_unfinalized_nonce();
}
