use codec::{Decode, Encode};
use log::*;
use polkadex_sgx_primitives::{AccountId, AssetId, PolkadexAccount};
use sgx_tstd::collections::HashMap;
use sgx_tstd::hash::Hash;
use sgx_tstd::hash::Hasher;
use sgx_tstd::vec::Vec;
use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use sp_core::blake2_256;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

static GLOBAL_POLKADEX_BALANCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub type EncodedKey = Vec<u8>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Balances {
    pub free: u128,
    pub reserved: u128,
}

impl Balances {
    pub fn from(free: u128, reserved: u128) -> Self {
        Self { free, reserved }
    }
}

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
        self.storage
            .get(&PolkadexBalanceKey::from(token, acc).encode())
    }

    pub fn set_free_balance(&mut self, token: AssetId, acc: AccountId, amt: u128) -> SgxResult<()> {
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
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
    }

    pub fn set_reserve_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: u128,
    ) -> SgxResult<()> {
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
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
    }

    pub fn deposit(&mut self, token: AssetId, acc: AccountId, amt: u128) -> SgxResult<()> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = balance.free.saturating_add(amt);
                Ok(())
            }
            None => {
                error!("Account Id or Asset Id not available [here]");
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
    }

    pub fn withdraw(&mut self, token: AssetId, acc: AccountId, amt: u128) -> SgxResult<()> {
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
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
    }
    // We can write functions which settle balances for two trades but we need to know the trade structure for it
}

pub fn create_in_memory_balance_storage() -> SgxResult<()> {
    let balances_storage = PolkadexBalanceStorage::create();
    let storage_ptr = Arc::new(SgxMutex::<PolkadexBalanceStorage>::new(balances_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_POLKADEX_BALANCE_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub fn load_balance_storage() -> SgxResult<&'static SgxMutex<PolkadexBalanceStorage>> {
    let ptr = GLOBAL_POLKADEX_BALANCE_STORAGE.load(Ordering::SeqCst)
        as *mut SgxMutex<PolkadexBalanceStorage>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}

pub fn deposit(main_acc: AccountId, token: AssetId, amt: u128) -> SgxResult<()> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = mutex.lock().unwrap();
    balance_storage.deposit(token, main_acc, amt)
}

pub fn withdraw(main_acc: AccountId, token: AssetId, amt: u128) -> SgxResult<()> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = mutex.lock().unwrap();
    match balance_storage.read_balance(token.clone(), main_acc.clone()) {
        Some(balance) => {
            if balance.free >= amt {
                balance_storage.withdraw(token, main_acc, amt)
            } else {
                error!("Balance is low");
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
        None => {
            error!("Account Id or Asset Id is not available");
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
    }
}

pub fn get_balances(main_acc: AccountId, token: AssetId) -> SgxResult<Balances> {
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = mutex.lock().unwrap();
    if let Some(balance) = balance_storage.read_balance(token, main_acc).cloned() {
        Ok(balance)
    } else {
        error!("Account Id or Asset Id is not available");
        Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
    }
}
