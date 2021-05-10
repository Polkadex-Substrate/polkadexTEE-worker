use sgx_tstd::collections::HashMap;
use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};
use substratee_node_primitives::AssetId;

static GLOBAL_POLKADEX_BALANCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct PolkadexBalanceStorage {
    /// map (tokenID, AccountID) -> (balance free, balance reserved)
    pub storage: HashMap<(AssetId, [u8; 32]), (u128, u128)>,
}

impl PolkadexBalanceStorage {
    pub fn create() -> PolkadexBalanceStorage {
        PolkadexBalanceStorage {
            storage: HashMap::new(),
        }
    }

    pub fn from_hashmap(hashmap: HashMap<(AssetId, [u8; 32]), (u128, u128)>) -> Self{
        Self{
            storage: hashmap
        }
    }

    pub fn read_balance(&self, token: AssetId, acc: [u8; 32]) -> Option<&(u128, u128)> {
        self.storage.get(&(token, acc))
    }

    pub fn set_free_balance(&mut self, token: AssetId, acc: [u8; 32], amt: u128) -> SgxResult<()> {
        match self.storage.get_mut(&(token, acc)) {
            Some(balance) => {
                balance.0 = amt;
                Ok(())
            }
            None => {
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
    }

    pub fn set_reserve_balance(
        &mut self,
        token: AssetId,
        acc: [u8; 32],
        amt: u128,
    ) -> SgxResult<()> {
        match self.storage.get_mut(&(token, acc)) {
            Some(balance) => {
                balance.1 = amt;
                Ok(())
            }
            None => {
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
    }

    pub fn deposit(&mut self, token: AssetId, acc: [u8; 32], amt: u128) -> SgxResult<()> {
        match self.storage.get_mut(&(token, acc)) {
            Some(balance) => {
                balance.0 = balance.0.saturating_add(amt);
                Ok(())
            }
            None => {
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
    }

    pub fn withdraw(&mut self, token: AssetId, acc: [u8; 32], amt: u128) -> SgxResult<()> {
        match self.storage.get_mut(&(token, acc)) {
            Some(balance) => {
                balance.0 = balance.0.saturating_sub(amt);
                Ok(())
            }
            None => {
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
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}

pub fn deposit(main_acc: [u8; 32], token: AssetId, amt: u128) -> SgxResult<()> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = mutex.lock().unwrap();
    balance_storage.deposit(token, main_acc, amt)
}

pub fn withdraw(main_acc: [u8; 32], token: AssetId, amt: u128) -> SgxResult<()> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = mutex.lock().unwrap();
    match balance_storage.read_balance(token.clone(), main_acc) {
        Some(balance) => {
            if balance.0 >= amt {
                balance_storage.withdraw(token, main_acc, amt)
            } else {
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
        None => {
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
    }
}
