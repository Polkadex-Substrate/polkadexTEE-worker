use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use std::collections::HashMap;
use std::string::String;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

use openfinex::types::Order;

static GLOBAL_ORDERBOOK_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());
pub struct OrderbookStorage {
    storage: HashMap<String, Order>,
}

impl OrderbookStorage {
    pub fn create() -> OrderbookStorage {
        OrderbookStorage {
            storage: HashMap::new(),
        }
    }

    /// Inserts a order_uid-order pair into the orderbook.
    /// If the orderbook did not have this order_uid present, [None] is returned.
    /// If the orderbook did have this order_uid present, the order is updated, and the old order is returned.
    pub fn add_order(&mut self, order_uid: String, order: Order) -> Option<Order> {
        self.storage.insert(order_uid, order)
    }

    /// Inserts a order_uid-order pair into the orderbook.
    /// If the orderbook did not have this order_uid present, [None] is returned.
    /// If the orderbook did have this order_uid present, the order is updated, and the old order is returned.
    pub fn set_order(&mut self, order_uid: String, order: Order) -> Option<Order> {
        self.storage.insert(order_uid, order)
    }

    /// Removes a order_uid from the orderbook,
    /// returning the value at the order_uid if the order_uid was previously in the map.
    pub fn remove_order(&mut self, order_uid: &String) -> Option<Order> {
        self.storage.remove(order_uid)
    }

    /// Returns a reference to the order corresponding to the order_uid.
    pub fn read_order(&self, order_uid: &String) -> Option<&Order> {
        self.storage.get(order_uid)
    }

    pub fn write_orderbook_to_db() -> SgxResult<()> {
        // TODO: Checkpoints and asynchronously writes the Orderbook via ocall to Permanent DB
        Ok(())
    }

    pub fn load_in_memory_orderbook_from_db() -> SgxResult<()> {
        // TODO: This functions loads the in memory orderbook storage from permanent storage
        Ok(())
    }
}

/// Creates a Static Atomic Pointer for Orderbook Storage
pub fn create_in_memory_orderbook_storage() -> SgxResult<()> {
    let orderbook = OrderbookStorage::create();
    let storage_ptr = Arc::new(SgxMutex::<OrderbookStorage>::new(orderbook));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ORDERBOOK_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

/// Loads and Returns Orderbook under mutex from Static Atomics Pointer
pub fn load_orderbook() -> SgxResult<&'static SgxMutex<OrderbookStorage>> {
    let ptr = GLOBAL_ORDERBOOK_STORAGE.load(Ordering::SeqCst) as *mut SgxMutex<OrderbookStorage>;
    if ptr.is_null() {
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}
