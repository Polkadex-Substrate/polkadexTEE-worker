use std::collections::HashMap;

use crate::constants::ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT;
use codec::Encode;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use crate::polkadex_db::{GeneralDB, PolkadexDBError};
use polkadex_sgx_primitives::types::SignedOrder;

static ORDERBOOK_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct OrderbookMirror {
    general_db: GeneralDB,
}

impl OrderbookMirror {
    pub fn write(&mut self, order_uid: Vec<u8>, signed_order: &SignedOrder) {
        self.general_db.write(order_uid, signed_order.encode());
    }

    pub fn _find(&self, k: Vec<u8>) -> Result<SignedOrder, PolkadexDBError> {
        println!("Searching for Key");
        match self.general_db._find(k) {
            Some(v) => match SignedOrder::from_vec(&v) {
                Ok(order) => {
                    println!("Found Key");
                    Ok(order)
                }
                Err(_) => {
                    println!("Unable to Deserialize");
                    Err(PolkadexDBError::UnableToDeseralizeValue)
                }
            },
            None => {
                println!("Key returns None");
                Err(PolkadexDBError::UnableToDeseralizeValue)
                //Fix: Change to correct error
            }
        }
    }

    pub fn _delete(&mut self, k: Vec<u8>) {
        self.general_db._delete(k);
    }

    pub fn read_all(&self) -> Result<Vec<SignedOrder>, PolkadexDBError> {
        let iterator = self.general_db.read_all().into_iter();
        let mut orders: Vec<SignedOrder> = vec![];
        for (_, value) in iterator.take(ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT) {
            match SignedOrder::from_vec(&*value) {
                Ok(order) => orders.push(order),
                Err(_) => {
                    println!("Unable to deserialize");
                    return Err(PolkadexDBError::UnableToDeseralizeValue);
                }
            }
        }
        Ok(orders)
    }
}

pub fn initialize_orderbook() {
    let storage_ptr = Arc::new(Mutex::<GeneralDB>::new(GeneralDB { db: HashMap::new() }));
    let ptr = Arc::into_raw(storage_ptr);
    ORDERBOOK_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
}

pub fn load_orderbook() -> Result<&'static Mutex<OrderbookMirror>, PolkadexDBError> {
    let ptr = ORDERBOOK_MIRROR.load(Ordering::SeqCst) as *mut Mutex<OrderbookMirror>;
    if ptr.is_null() {
        println!("Unable to load the pointer");
        Err(PolkadexDBError::UnableToLoadPointer)
    } else {
        Ok(unsafe { &*ptr })
    }
}
