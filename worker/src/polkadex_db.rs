use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;
use std::thread::JoinHandle;

use codec::Encode;
use log::error;
use rocksdb::{DB, DBWithThreadMode, Error as RocksDBError, Error, IteratorMode, Options, SingleThreaded};

use polkadex_primitives::types::{Order, SignedOrder};

///
/// Polkadex Orderbook Mirror Documentation
/// The backend DB is RocksDb
/// Orders are stored as (OrderUUID,SignedOrder)
/// where SignedOrder contains Order, counter and signature of enclave
///

use crate::constants::{ORDERBOOK_DB_FILE, ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT};

static ORDERBOOK_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct RocksDB {
    pub(crate) db: DBWithThreadMode<SingleThreaded>,
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
    fn initialize_db(create_if_missing_db: bool) -> Result<(), RocksDBError>;
    fn load_orderbook_mirror() -> Result<&'static Mutex<RocksDB>, PolkadexDBError>;
    fn write(db: &MutexGuard<RocksDB>, order_uid: Vec<u8>, signed_order: &SignedOrder) -> Result<(), PolkadexDBError>;
    fn find(k: Vec<u8>) -> Result<Option<SignedOrder>, PolkadexDBError>;
    fn delete(db: &MutexGuard<RocksDB>,k: Vec<u8>) -> Result<(), PolkadexDBError>;
    fn read_all() -> Result<Vec<SignedOrder>, PolkadexDBError>;
}

impl KVStore for RocksDB {
    fn initialize_db(create_if_missing_db: bool) -> Result<(), RocksDBError> {
        let mut opts = Options::default();
        opts.create_if_missing(create_if_missing_db);

        let db = DB::open(&opts, ORDERBOOK_DB_FILE)?;
        let storage_ptr = Arc::new(Mutex::<RocksDB>::new(RocksDB { db }));
        let ptr = Arc::into_raw(storage_ptr);
        // FIXME: Do we really need SeqCst here?, RocksDB already takes care of concurrent writes.
        ORDERBOOK_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
        Ok(())
    }

    fn load_orderbook_mirror() -> Result<&'static Mutex<RocksDB>, PolkadexDBError> {
        let ptr = ORDERBOOK_MIRROR.load(Ordering::SeqCst) as *mut Mutex<RocksDB>;
        if ptr.is_null() {
            println!(" Unable to load the pointer");
            return Err(PolkadexDBError::UnableToLoadPointer);
        } else {
            Ok(unsafe { &*ptr })
        }
    }
    fn write(db: &MutexGuard<RocksDB>, order_uid: Vec<u8>, signed_order: &SignedOrder) -> Result<(), PolkadexDBError>{
        match db.db.put(order_uid, signed_order.encode()) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!(" Error {} writing to DB", e);
                Err(PolkadexDBError::ErrorWritingToDB)
            }
        }
    }

    fn find(k: Vec<u8>) -> Result<Option<SignedOrder>, PolkadexDBError> {
        let mutex = RocksDB::load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
        println!("Searching for Key");
        match orderbook_mirror.db.get(k) {
            Ok(Some(mut v)) => {
                match SignedOrder::from_vec(&mut v.as_mut()) {
                    Ok(order) => {
                        println!("Found Key");
                        Ok(Some(order))
                    }
                    Err(e) => {
                        println!("Unable to Deserialize ");
                        Err(PolkadexDBError::UnableToDeseralizeValue)
                    }
                }
            }
            Ok(None) => {
                println!("Key returns None");
                Ok(None)
            }
            Err(e) => {
                println!("Error retrieving value: {}", e);
                Err(PolkadexDBError::UnableToRetrieveValue)
            }
        }
    }

    fn delete(db: &MutexGuard<RocksDB>,k: Vec<u8>) -> Result<(), PolkadexDBError> {
        match db.db.delete(k) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Error Deleteing Key, {}", e);
                Err(PolkadexDBError::ErrorDeleteingKey)
            }
        }
    }

    fn read_all() -> Result<Vec<SignedOrder>, PolkadexDBError> {
        let mutex = RocksDB::load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
        let iterator = orderbook_mirror.db.iterator(IteratorMode::Start);
        let mut orders: Vec<SignedOrder> = vec![];
        for (_, value) in iterator.take(ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT) {
            match SignedOrder::from_vec(&*value) {
                Ok(order) => { orders.push(order) }
                Err(e) => {
                    println!("Unable to deserialize ");
                    return Err(PolkadexDBError::UnableToDeseralizeValue);
                }
            }
        }
        Ok(orders)
    }
}


