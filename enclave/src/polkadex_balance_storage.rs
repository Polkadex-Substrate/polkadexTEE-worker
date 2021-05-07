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
    storage: HashMap<(AssetId, [u8; 32]), (u128, u128)>,
}

impl PolkadexBalanceStorage {
    pub fn create() -> PolkadexBalanceStorage {
        PolkadexBalanceStorage {
            storage: HashMap::new(),
        }
    }

    pub fn read_balance(&self, token: AssetId, acc: [u8; 32]) -> (u128, u128) {
        self.storage.get((token, acc)).unwrap()
    }

    pub fn read_free_balance(&self, token: AssetId, acc: [u8; 32]) -> u128 {
        self.storage.get((token, acc)).unwrap().0
    }
    pub fn read_reserve_balance(&self, token: AssetId, acc: [u8; 32]) -> u128 {
        self.storage.get((token, acc)).unwrap().1
    }

    pub fn set_free_balance(&mut self, token: AssetId, acc: [u8; 32], amt: u128) -> SgxResult<()> {
        let balance = self.storage.get_mut((token, acc)).unwrap();
        balance.0 = amt;
        Ok(())
    }

    pub fn set_reserve_balance(
        &mut self,
        token: AssetId,
        acc: [u8; 32],
        amt: u128,
    ) -> SgxResult<()> {
        let balance = self.storage.get_mut((token, acc)).unwrap();
        balance.1 = amt;
        Ok(())
    }

    pub fn deposit(&mut self, token: AssetId, acc: [u8; 32], amt: u128) -> SgxResult<()> {
        let balance = self.storage.get_mut((token, acc)).unwrap();
        balance.0 = balance.0 + amt; // TODO: Handle Overflow
        Ok(())
    }

    pub fn withdraw(&mut self, token: AssetId, acc: [u8; 32], amt: u128) -> SgxResult<()> {
        let balance = self.storage.get_mut((token, acc)).unwrap();
        balance.0 = balance.0 - amt; // TODO: Handle Underflow
        Ok(())
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

pub fn load_proxy_registry() -> SgxResult<&'static SgxMutex<PolkadexBalanceStorage>> {
    let ptr = GLOBAL_POLKADEX_BALANCE_STORAGE.load(Ordering::SeqCst)
        as *mut SgxMutex<PolkadexBalanceStorage>;
    if ptr.is_null() {
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}
