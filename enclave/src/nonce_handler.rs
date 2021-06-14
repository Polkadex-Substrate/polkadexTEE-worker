use log::*;
use sgx_types::{sgx_status_t, SgxResult};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

static GLOBAL_POLKADEX_NONCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct NonceHandler {
    pub nonce: u32,
    pub is_initialized: bool,
}

impl NonceHandler {
    pub fn create() -> Self {
        Self {
            nonce: 0u32, //We can also use option
            is_initialized: false,
        }
    }

    pub fn increment(&mut self) {
        self.nonce += 1;
    }
}

pub fn create_in_memory_nonce_storage() -> SgxResult<()> {
    let nonce_storage = NonceHandler::create();
    let storage_ptr = Arc::new(SgxMutex::<NonceHandler>::new(nonce_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_POLKADEX_NONCE_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub fn load_nonce_storage() -> SgxResult<&'static SgxMutex<NonceHandler>> {
    let ptr = GLOBAL_POLKADEX_NONCE_STORAGE.load(Ordering::SeqCst) as *mut SgxMutex<NonceHandler>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}

pub fn lock_and_update_nonce(nonce: u32) -> SgxResult<()> {
    let mutex = load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<NonceHandler> = mutex.lock().unwrap();
    if let false = nonce_storage.is_initialized {
        nonce_storage.nonce = nonce;
        nonce_storage.is_initialized = true;
        Ok(())
    } else {
        Ok(())
    }
}
