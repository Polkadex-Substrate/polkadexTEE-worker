// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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

    pub fn update(&mut self, nonce: u32) {
        self.nonce = nonce;
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
    debug!("update to new nonce: {:?}", nonce);
    if let false = nonce_storage.is_initialized {
        nonce_storage.nonce = nonce;
        nonce_storage.is_initialized = true;
        Ok(())
    } else {
        Ok(())
    }
}
