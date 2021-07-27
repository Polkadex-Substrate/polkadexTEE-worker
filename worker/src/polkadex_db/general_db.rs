use std::collections::HashMap;

use crate::constants::{ORDERBOOK_DB_FILE, ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use codec::Encode;
use rocksdb::{DBWithThreadMode, Error as RocksDBError, IteratorMode, Options, SingleThreaded, DB};

use polkadex_sgx_primitives::types::SignedOrder;

static ORDERBOOK_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct GeneralDB {
    pub db: HashMap<Vec<u8>, Vec<u8>>,
}

pub enum PolkadexDBError {
    UnableToLoadPointer,
    UnableToRetrieveValue,
    ErrorWritingToDB,
    UnableToDeseralizeValue,
    ErrorDeleteingKey,
}

pub trait KVStore {
    /// Loads the DB from file on disk
    fn initialize_db();
    fn load_orderbook_mirror() -> Result<&'static Mutex<GeneralDB>, PolkadexDBError>;
    fn write(
        db: &mut MutexGuard<GeneralDB>,
        order_uid: Vec<u8>,
        signed_order: &SignedOrder,
    ) -> Result<(), PolkadexDBError>;
    fn find(k: Vec<u8>) -> Result<Option<SignedOrder>, PolkadexDBError>;
    fn delete(db: &mut MutexGuard<GeneralDB>, k: Vec<u8>) -> Result<(), PolkadexDBError>;
    fn read_all() -> Result<Vec<SignedOrder>, PolkadexDBError>;
}

impl KVStore for GeneralDB {
    fn initialize_db() {
        let storage_ptr = Arc::new(Mutex::<GeneralDB>::new(GeneralDB { db: HashMap::new() }));
        let ptr = Arc::into_raw(storage_ptr);
        // FIXME: Do we really need SeqCst here?, RocksDB already takes care of concurrent writes.
        ORDERBOOK_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
    }

    fn load_orderbook_mirror() -> Result<&'static Mutex<GeneralDB>, PolkadexDBError> {
        let ptr = ORDERBOOK_MIRROR.load(Ordering::SeqCst) as *mut Mutex<GeneralDB>;
        if ptr.is_null() {
            println!("Unable to load the pointer");
            Err(PolkadexDBError::UnableToLoadPointer)
        } else {
            Ok(unsafe { &*ptr })
        }
    }
    fn write(
        db: &mut MutexGuard<GeneralDB>,
        order_uid: Vec<u8>,
        signed_order: &SignedOrder,
    ) -> Result<(), PolkadexDBError> {
        db.db.insert(order_uid, signed_order.encode());
        Ok(())
    }

    fn find(k: Vec<u8>) -> Result<Option<SignedOrder>, PolkadexDBError> {
        let mutex = GeneralDB::load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<GeneralDB> = mutex.lock().unwrap();
        println!("Searching for Key");
        match orderbook_mirror.db.get(&k) {
            Some(v) => match SignedOrder::from_vec(&v) {
                Ok(order) => {
                    println!("Found Key");
                    Ok(Some(order))
                }
                Err(_) => {
                    println!("Unable to Deserialize ");
                    Err(PolkadexDBError::UnableToDeseralizeValue)
                }
            },
            None => {
                println!("Key returns None");
                Ok(None)
            }
        }
    }

    fn delete(db: &mut MutexGuard<GeneralDB>, k: Vec<u8>) -> Result<(), PolkadexDBError> {
        db.db.remove(&k);
        Ok(())
    }

    fn read_all() -> Result<Vec<SignedOrder>, PolkadexDBError> {
        let mutex = GeneralDB::load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<GeneralDB> = mutex.lock().unwrap();
        let iterator = orderbook_mirror.db.iter();
        let mut orders: Vec<SignedOrder> = vec![];
        for (_, value) in iterator.take(ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT) {
            match SignedOrder::from_vec(&*value) {
                Ok(order) => orders.push(order),
                Err(_) => {
                    println!("Unable to deserialize ");
                    return Err(PolkadexDBError::UnableToDeseralizeValue);
                }
            }
        }
        Ok(orders)
    }
}
