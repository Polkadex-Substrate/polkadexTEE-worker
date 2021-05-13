use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;

use log::error;
use rocksdb::{DB, DBWithThreadMode, Error, IteratorMode, Options, SingleThreaded};

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
use codec::Encode;

static ORDERBOOK_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct RocksDB {
    db: DBWithThreadMode<SingleThreaded>,
}

pub trait KVStore {
    /// Loads the DB from file on disk
    fn initialize_db(create_if_missing_db: bool) -> Result<(), Error>;
    fn load_orderbook_mirror() -> Result<&Mutex<RocksDB>, ()>;
    fn write(order_uid: &str, signed_order: SignedOrder);
    fn find(&self, k: &str) -> Option<SignedOrder>;
    fn delete(k: &str);
}

impl KVStore for RocksDB {
    fn initialize_db(create_if_missing_db: bool) -> Result<(), Error> {
        let mut opts = Options::default();
        opts.create_if_missing(create_if_missing_db);

        let db = DB::open(&opts, ORDERBOOK_DB_FILE)?;
        let storage_ptr = Arc::new(Mutex::<RocksDB>::new(RocksDB { db }));
        let ptr = Arc::into_raw(storage_ptr);
        // FIXME: Do we really need SeqCst here?, RocksDB already takes care of concurrent writes.
        ORDERBOOK_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
        Ok(())
    }

    fn load_orderbook_mirror() -> Result<&Mutex<RocksDB>, ()> {
        let ptr = ORDERBOOK_MIRROR.load(Ordering::SeqCst) as *mut Mutex<RocksDB>;
        if ptr.is_null() {
            error!(" Unable to load the pointer");
            return Err(());
        } else {
            Ok(unsafe { &*ptr })
        }
    }
    fn write(order_uid: &str, signed_order: SignedOrder) {
        thread::spawn(||-> Result<(),()> {
            let mutex = RocksDB::load_orderbook_mirror()?;
            let mut orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
            orderbook_mirror.db.put(order_uid.as_bytes(), signed_order.encode()).is_ok();
            Ok(())
        });
    }

    fn find(&self, k: &str) -> Option<SignedOrder> {
        match self.db.get(k.as_bytes()) {
            Ok(Some(v)) => {
                let result = SignedOrder::from_vec(v.to_vec());
                Some(result)
            }
            Ok(None) => {
                println!("Finding '{}' returns None", k);
                None
            }
            Err(e) => {
                println!("Error retrieving value for {}: {}", k, e);
                None
            }
        }
    }

    fn delete(k: &str) {
        thread::spawn(||->Result<(),()> {
            let mutex = RocksDB::load_orderbook_mirror()?;
            let mut orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
            orderbook_mirror.db.delete(k.as_bytes()).is_ok();
            Ok(())
        });
    }
}


