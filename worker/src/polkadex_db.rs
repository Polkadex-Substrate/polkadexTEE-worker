use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;

use codec::Encode;
use log::error;
use rocksdb::{DB, DBWithThreadMode, Error as RocksDBError, IteratorMode, Options, SingleThreaded};

use polkadex_primitives::types::{Order, SignedOrder};

///
/// Polkadex Orderbook Mirror Documentation
/// The backend DB is RocksDb
/// Orders are stored as (counter,SignedOrder)
/// where SignedOrder contains Order, counter and signature of enclave
///
///
///
///

use crate::constants::ORDERBOOK_DB_FILE;
use std::thread::JoinHandle;

static ORDERBOOK_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct RocksDB {
    db: DBWithThreadMode<SingleThreaded>,
}

pub enum PolkadexDBError {
    UnableToLoadPointer,
    UnableToRetrieveValue,
}

pub trait KVStore {
    /// Loads the DB from file on disk
    fn initialize_db(create_if_missing_db: bool) -> Result<(), RocksDBError>;
    fn load_orderbook_mirror() -> Result<&'static Mutex<RocksDB>, PolkadexDBError>;
    fn write(order_uid: &'static str, signed_order: SignedOrder) -> JoinHandle<Result<(), PolkadexDBError>>;
    fn find(k: &str) -> Result<Option<SignedOrder>, PolkadexDBError>;
    fn delete(k: &'static str)  -> JoinHandle<Result<(), PolkadexDBError>>;
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
    fn write(order_uid: &'static str, signed_order: SignedOrder) -> JoinHandle<Result<(), PolkadexDBError>> {
        thread::spawn(move || -> Result<(), PolkadexDBError> {
            println!("Reached here!");
            let mutex = RocksDB::load_orderbook_mirror()?;
            println!("Reached here!");
            let mut orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
            println!("Reached here!");
            orderbook_mirror.db.put(order_uid.as_bytes(), signed_order.encode()).is_ok();
            Ok(())
        })
    }

    fn find(k: &str) -> Result<Option<SignedOrder>, PolkadexDBError> {
        let mutex = RocksDB::load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
        match orderbook_mirror.db.get(k.as_bytes()) {
            Ok(Some(v)) => {
                let result = SignedOrder::from_vec(v.to_vec());
                Ok(Some(result))
            }
            Ok(None) => {
                println!("Finding '{}' returns None", k);
                Ok(None)
            }
            Err(e) => {
                println!("Error retrieving value for {}: {}", k, e);
                Err(PolkadexDBError::UnableToRetrieveValue)
            }
        }
    }

    fn delete(k: &'static str)  -> JoinHandle<Result<(), PolkadexDBError>> {
        thread::spawn(move || -> Result<(), PolkadexDBError> {
            let mutex = RocksDB::load_orderbook_mirror()?;
            let mut orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
            orderbook_mirror.db.delete(k.as_bytes()).is_ok();
            Ok(())
        })
    }
}


