use crate::ed25519;
use crate::polkadex_gateway::GatewayError;
use log::error;
use polkadex_sgx_primitives::types::{Order, OrderUUID, SignedOrder};
use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use sp_core::{ed25519::Signature, Pair};
use std::collections::HashMap;
use std::string::String;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};
use std::vec::Vec;

static GLOBAL_ORDERBOOK_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct OrderbookStorage {
    storage: HashMap<OrderUUID, Order>,
}

impl OrderbookStorage {
    pub fn create(verified_orders: Vec<SignedOrder>) -> OrderbookStorage {
        let mut storage: HashMap<OrderUUID, Order> = HashMap::new();

        for order in verified_orders {
            storage.insert(order.order_id, order.order);
        }

        OrderbookStorage { storage }
    }

    /// Inserts a order_uid-order pair into the orderbook.
    /// If the orderbook did not have this order_uid present, [None] is returned.
    /// If the orderbook did have this order_uid present, the order is updated, and the old order is returned.
    pub fn add_order(&mut self, order_uid: OrderUUID, order: Order) -> Option<Order> {
        self.storage.insert(order_uid, order)
    }

    /// Removes a order_uid from the orderbook,
    /// returning the value at the order_uid if the order_uid was previously in the map.
    pub fn remove_order(&mut self, order_uid: &OrderUUID) -> Option<Order> {
        self.storage.remove(order_uid)
    }

    /// Returns a reference to the order corresponding to the order_uid.
    pub fn read_order(&self, order_uid: &OrderUUID) -> Option<&Order> {
        self.storage.get(order_uid)
    }

    pub fn write_orderbook_to_db(order_id: OrderUUID, order: Order) -> SgxResult<()> {
        let signer_pair = ed25519::unseal_pair()?;
        let mut signed_order = SignedOrder {
            order_id,
            order,
            signature: Signature::default(),
        };
        signed_order.sign(&signer_pair);
        crate::write_order_to_disk(signed_order);
        Ok(())
    }
}

/// Creates a Static Atomic Pointer for Orderbook Storage
pub fn create_in_memory_orderbook_storage(signed_orders: Vec<SignedOrder>) -> SgxResult<()> {
    let mut verified_orders: Vec<SignedOrder> = vec![];
    let signer_pair = ed25519::unseal_pair()?;
    for order in signed_orders {
        if !order.verify_signature(&signer_pair) {
            error!("Signature Verification Failed");
            continue;
        }
        verified_orders.push(order)
    }
    let orderbook = OrderbookStorage::create(verified_orders);
    let storage_ptr = Arc::new(SgxMutex::<OrderbookStorage>::new(orderbook));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ORDERBOOK_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

/// Loads and Returns Orderbook under mutex from Static Atomics Pointer
pub fn load_orderbook() -> Result<&'static SgxMutex<OrderbookStorage>, GatewayError> {
    let ptr = GLOBAL_ORDERBOOK_STORAGE.load(Ordering::SeqCst) as *mut SgxMutex<OrderbookStorage>;
    if ptr.is_null() {
        return Err(GatewayError::UnableToLoadPointer);
    } else {
        Ok(unsafe { &*ptr })
    }
}

// TODO: Write test cases for this function

pub fn lock_storage_and_remove_order(order_uuid: &OrderUUID) -> Result<Order, GatewayError> {
    let mutex = load_orderbook()?;
    // TODO: Handle this unwrap
    let mut orderbook: SgxMutexGuard<OrderbookStorage> = mutex.lock().unwrap();
    if let Some(order) = orderbook.remove_order(order_uuid) {
        Ok(order)
    } else {
        Err(GatewayError::OrderNotFound)
    }
}

// TODO: Write test cases for this function


pub fn lock_storage_and_add_order(
    order: Order,
    order_uuid: OrderUUID,
) -> Result<Option<Order>, GatewayError> {
    let mutex = load_orderbook()?;
    // TODO: Handle this unwrap
    let mut orderbook: SgxMutexGuard<OrderbookStorage> = mutex.lock().unwrap();
    Ok(orderbook.add_order(order_uuid, order))
}


pub fn lock_storage_and_check_order_in_orderbook(
    order_uuid: OrderUUID,
) -> Result<bool, GatewayError> {
    let mutex = load_orderbook().unwrap();
    let mut orderbook: SgxMutexGuard<OrderbookStorage> = mutex.lock().unwrap();
    Ok(orderbook.storage.contains_key(&order_uuid))
}

// Only for test
pub fn lock_storage_and_get_order(order_uuid: OrderUUID) -> Result<Order, GatewayError> {
    let mutex = load_orderbook()?;
    let mut orderbook: SgxMutexGuard<OrderbookStorage> = mutex.lock().unwrap();
    let order = orderbook.read_order(&order_uuid).unwrap().clone();
    Ok(order)
}
