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

use codec::Encode;
use log::*;
use polkadex_sgx_primitives::AccountId;
use sgx_tstd::collections::HashMap;
use sgx_tstd::vec::Vec;

pub type EncodedKey = Vec<u8>;

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

    pub fn read_nonce(&self, acc: AccountId) -> Result<u32, NonceStorageError> {
        debug!("reading nonce from acc: {:?}", acc);
        if let Some(nonce) = self.storage.get(&acc.encode()) {
            Ok(*nonce)
        } else {
            error!("Nonce uninitialized");
            Err(NonceStorageError::NonceUninitialized)
        }
    }

    pub fn increment_nonce(&mut self, acc: AccountId) -> Result<(), NonceStorageError> {
        let nonce = self.read_nonce(acc.clone())?;
        self.storage.insert(acc.encode(), nonce + 1u32);
        Ok(())
    }

    pub fn initialize_nonce(&mut self, acc: AccountId) {
        debug!("initializing nonce for acc: {:?}", acc);
        self.storage.insert(acc.encode(), 0u32);
    }

    pub fn remove_nonce(&mut self, acc: AccountId) {
        debug!("initializing nonce for acc: {:?}", acc);
        self.storage.remove(&acc.encode());
    }
}

#[derive(Eq, Debug, PartialEq, PartialOrd)]
pub enum NonceStorageError {
    /// Nonce is not initialized
    NonceUninitialized,
}

pub mod tests {
    use crate::nonce_storage::PolkadexNonceStorage;
    use sp_core::{ed25519 as ed25519_core, Pair};

    pub fn nonce_initialized_correctly() {
        let account_id =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012").public();
        let mut nonce_storage: PolkadexNonceStorage = PolkadexNonceStorage::create();
        nonce_storage.initialize_nonce(account_id.into());

        assert_eq!(Ok(0u32), nonce_storage.read_nonce(account_id.into()));
    }

    pub fn nonce_incremented_correctly() {
        let account_id =
            ed25519_core::Pair::from_seed(b"12345678901234567890123456789012").public();
        let mut nonce_storage: PolkadexNonceStorage = PolkadexNonceStorage::create();
        nonce_storage.initialize_nonce(account_id.into());

        assert_eq!(Ok(0u32), nonce_storage.read_nonce(account_id.into()));

        nonce_storage.increment_nonce(account_id.into()).unwrap();

        assert_eq!(Ok(1u32), nonce_storage.read_nonce(account_id.into()));

        nonce_storage.increment_nonce(account_id.into()).unwrap();
        nonce_storage.increment_nonce(account_id.into()).unwrap();
        nonce_storage.increment_nonce(account_id.into()).unwrap();

        assert_eq!(Ok(4u32), nonce_storage.read_nonce(account_id.into()));
    }
}
