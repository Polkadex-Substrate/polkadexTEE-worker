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
use codec::Encode;
use log::*;
use polkadex_sgx_primitives::AccountId;
use sgx_tstd::collections::HashMap;
use sgx_tstd::vec::Vec;
use sgx_types::{sgx_status_t, SgxResult};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

pub type EncodedKey = Vec<u8>;

static GLOBAL_POLKADEX_USER_NONCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[derive(Debug)]
pub struct PolkadexNonceStorage {
    /// map AccountId -> NonceHandler
    pub storage: HashMap<EncodedKey, u32>,
}

impl PolkadexNonceStorage {
    pub fn create() -> PolkadexNonceStorage {
        PolkadexNonceStorage {
            storage: HashMap::new(),
        }
    }

    pub fn read_nonce(&mut self, acc: AccountId) -> u32 {
        debug!("reading nonce from acc: {:?}", acc);
        if let Some(nonce) = self.storage.get(&acc.encode()) {
            *nonce
        } else {
            self.initialize_nonce(acc);
            0u32
        }
    }

    pub fn increment_nonce(&mut self, acc: AccountId) {
        let nonce = self.read_nonce(acc.clone());
        self.storage.insert(acc.encode(), nonce + 1u32);
    }

    pub fn initialize_nonce(&mut self, acc: AccountId) {
        debug!("initializing nonce for acc: {:?}", acc);
        self.storage.insert(acc.encode(), 0u32);
    }
}

pub fn create_in_memory_nonce_storage() -> Result<(), GatewayError> {
    let nonce_storage = PolkadexNonceStorage::create();
    let storage_ptr = Arc::new(SgxMutex::<PolkadexNonceStorage>::new(nonce_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_POLKADEX_USER_NONCE_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub fn load_nonce_storage() -> SgxResult<&'static SgxMutex<PolkadexNonceStorage>> {
    let ptr = GLOBAL_POLKADEX_USER_NONCE_STORAGE.load(Ordering::SeqCst)
        as *mut SgxMutex<PolkadexNonceStorage>;
    if ptr.is_null() {
        error!("Pointer is Null");
        Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
    } else {
        Ok(unsafe { &*ptr })
    }
}

pub fn lock_storage_and_get_nonce(acc: AccountId) -> SgxResult<u32> {
    let mutex = load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<PolkadexNonceStorage> = mutex.lock().unwrap(); //TODO: Error handling
    let nonce = nonce_storage.read_nonce(acc);
    Ok(nonce)
}

pub fn lock_storage_and_increment_nonce(acc: AccountId) -> SgxResult<()> {
    let mutex = load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<PolkadexNonceStorage> = mutex.lock().unwrap();
    nonce_storage.increment_nonce(acc);
    Ok(())
}

pub mod tests {
    use crate::nonce_storage::PolkadexNonceStorage;
    use sp_core::{ed25519 as ed25519_core, Pair, H256};

    pub fn nonce_initialized_correctly() {
        let account_id =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012").public();
        let mut nonce_storage: PolkadexNonceStorage = PolkadexNonceStorage::create();
        nonce_storage.initialize_nonce(account_id.into());

        assert_eq!(0u32, nonce_storage.read_nonce(account_id.into()));
    }

    pub fn nonce_incremented_correctly() {
        let account_id =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012").public();
        let mut nonce_storage: PolkadexNonceStorage = PolkadexNonceStorage::create();
        nonce_storage.initialize_nonce(account_id.into());

        assert_eq!(0u32, nonce_storage.read_nonce(account_id.into()));

        nonce_storage.increment_nonce(account_id.into());

        assert_eq!(1u32, nonce_storage.read_nonce(account_id.into()));

        nonce_storage.increment_nonce(account_id.into());
        nonce_storage.increment_nonce(account_id.into());
        nonce_storage.increment_nonce(account_id.into());

        assert_eq!(4u32, nonce_storage.read_nonce(account_id.into()));
    }
}
