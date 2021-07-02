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

use crate::polkadex_gateway::GatewayError;
use polkadex_sgx_primitives::AccountId;
use log::*;
use sgx_types::{sgx_status_t, SgxResult};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

pub mod nonce_storage;
pub mod nonce_handler;

pub use nonce_storage::*;
pub use nonce_handler::*;

static GLOBAL_POLKADEX_NONCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn create_in_memory_nonce_storage() -> Result<(), GatewayError> {
    let nonce_storage = PolkadexNonceStorage::create();
    let storage_ptr = Arc::new(SgxMutex::<PolkadexNonceStorage>::new(nonce_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_POLKADEX_NONCE_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub fn load_nonce_storage() -> SgxResult<&'static SgxMutex<PolkadexNonceStorage>> {
    let ptr = GLOBAL_POLKADEX_NONCE_STORAGE.load(Ordering::SeqCst)
        as *mut SgxMutex<PolkadexNonceStorage>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }
    Ok(unsafe { &*ptr })
}

pub fn lock_storage_and_get_nonce(
    main_acc: AccountId,
) -> SgxResult<NonceHandler> {
    let mutex = load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<PolkadexNonceStorage> = mutex.lock()
        .unwrap(); //TODO: Error handling
    if let Some(nonce) = nonce_storage.read_nonce(main_acc.clone()).cloned() {
        Ok(nonce)
    } else {
        nonce_storage.initialize_nonce(main_acc.clone());
        Ok(NonceHandler {nonce: Some(0u32)})
    }
}

pub fn lock_and_update_nonce(nonce: u32, acc: AccountId) -> SgxResult<()> {
    let mutex = load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<PolkadexNonceStorage> = mutex.lock().unwrap();
    nonce_storage.set_nonce(nonce, acc);
    Ok(())
}

pub fn lock_storage_and_increment_nonce(acc: AccountId) -> SgxResult<()> {
    let mutex = load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<PolkadexNonceStorage> = mutex.lock().unwrap();
    if let Some(mut nonce_handler) = nonce_storage.read_nonce(acc.clone()).cloned() {
        if let Some(inner_nonce) = nonce_handler.nonce {
            nonce_storage.set_nonce(inner_nonce + 1u32, acc);
            Ok(())
        } else {
            nonce_handler.nonce = Some(0u32);
            Ok(())
        }
    } else {
        nonce_storage.initialize_nonce(acc.clone());
        Ok(())
    }

}