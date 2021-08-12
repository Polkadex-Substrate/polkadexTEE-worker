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

use std::collections::HashMap;

use crate::polkadex_db::{GeneralDB, PolkadexDBError};
use codec::{Decode, Encode};
use polkadex_sgx_primitives::AccountId;
use std::path::PathBuf;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use super::disk_storage_handler::DiskStorageHandler;
use super::PermanentStorageHandler;
use super::Result;
use crate::constants::NONCE_DISK_STORAGE_FILENAME;

static NONCE_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[derive(Debug)]
pub struct NonceMirror<D: PermanentStorageHandler> {
    general_db: GeneralDB<D>,
}

#[derive(Encode, Decode)]
struct Nonce {
    nonce: u32,
}

impl<D: PermanentStorageHandler> NonceMirror<D> {
    pub fn write(&mut self, account_id: AccountId, nonce: u32) {
        self.general_db
            .write(account_id.encode(), Nonce { nonce }.encode());
    }

    pub fn _find(&self, k: AccountId) -> Result<u32> {
        println!("Searching for Key");
        match self.general_db._find(k.encode()) {
            Some(v) => Ok(Nonce::decode(&mut v.as_slice())
                .map_err(|_| PolkadexDBError::_DecodeError)?
                .nonce),
            None => {
                println!("Key returns None");
                Err(PolkadexDBError::_KeyNotFound)
            }
        }
    }

    pub fn _delete(&mut self, k: AccountId) {
        self.general_db._delete(k.encode());
    }

    pub fn take_disk_snapshot(&mut self) -> Result<()> {
        self.general_db.write_disk_from_memory()
    }
}

pub fn initialize_nonce_mirror() {
    let storage_ptr = Arc::new(Mutex::<NonceMirror<DiskStorageHandler>>::new(NonceMirror {
        general_db: GeneralDB::new(
            HashMap::new(),
            DiskStorageHandler::open_default(PathBuf::from(NONCE_DISK_STORAGE_FILENAME)),
        ),
    }));
    let ptr = Arc::into_raw(storage_ptr);
    NONCE_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
}

pub fn load_nonce_mirror() -> Result<&'static Mutex<NonceMirror<DiskStorageHandler>>> {
    let ptr = NONCE_MIRROR.load(Ordering::SeqCst) as *mut Mutex<NonceMirror<DiskStorageHandler>>;
    if ptr.is_null() {
        println!("Unable to load the pointer");
        Err(PolkadexDBError::UnableToLoadPointer)
    } else {
        Ok(unsafe { &*ptr })
    }
}

#[cfg(test)]
mod tests {
    use super::GeneralDB;
    use crate::polkadex_db::mock::PermanentStorageMock;
    use crate::polkadex_db::NonceMirror;
    use codec::Encode;
    use polkadex_primitives::AccountId;
    use sp_core::{ed25519 as ed25519_core, Pair};
    use std::collections::HashMap;

    fn create_dummy_account() -> AccountId {
        AccountId::from(ed25519_core::Pair::from_seed(b"12345678901234567890123456789012").public())
    }
    fn create_secondary_dummy_account() -> AccountId {
        AccountId::from(ed25519_core::Pair::from_seed(b"01234567890123456789012345678901").public())
    }

    #[test]
    fn write() {
        let dummy_account = create_dummy_account();
        let mut nonce_mirror = NonceMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        assert_eq!(nonce_mirror.general_db.db, HashMap::new());
        nonce_mirror.write(dummy_account.clone(), 42u32);
        assert_eq!(
            nonce_mirror.general_db.db.get(&dummy_account.encode()),
            Some(&42u32.encode())
        );
    }

    #[test]
    fn find() {
        let dummy_account = create_dummy_account();
        let mut nonce_mirror = NonceMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        nonce_mirror
            .general_db
            .db
            .insert(dummy_account.encode(), 42u32.encode());
        assert_eq!(nonce_mirror._find(dummy_account).unwrap(), 42u32);
        assert!(nonce_mirror
            ._find(create_secondary_dummy_account())
            .is_err());
    }

    #[test]
    fn delete() {
        let dummy_account = create_dummy_account();
        let mut nonce_mirror = NonceMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        nonce_mirror
            .general_db
            .db
            .insert(dummy_account.encode(), 42u32.encode());
        assert!(nonce_mirror
            .general_db
            .db
            .contains_key(&dummy_account.encode()));
        nonce_mirror._delete(dummy_account.clone());
        assert!(!nonce_mirror
            .general_db
            .db
            .contains_key(&dummy_account.encode()));
    }
}
