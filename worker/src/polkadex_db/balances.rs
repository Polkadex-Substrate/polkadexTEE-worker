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

use codec::{Decode, Encode};
use std::path::PathBuf;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use crate::polkadex_db::{GeneralDB, PolkadexDBError};
use polkadex_sgx_primitives::{AccountId, AssetId};

use super::disk_storage_handler::DiskStorageHandler;
use super::PermanentStorageHandler;
use super::Result;
use crate::constants::BALANCE_DISK_STORAGE_FILENAME;
use polkadex_sgx_primitives::BalancesData;

static BALANCES_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[derive(Debug)]
pub struct BalancesMirror<D: PermanentStorageHandler> {
    general_db: GeneralDB<D>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Balances {
    free: u128,
    reserved: u128,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PolkadexBalanceKey {
    asset_id: AssetId,
    account_id: AccountId,
}

impl PolkadexBalanceKey {
    pub fn from(asset_id: AssetId, account_id: AccountId) -> Self {
        Self {
            asset_id,
            account_id,
        }
    }
}

impl<D: PermanentStorageHandler> BalancesMirror<D> {
    pub fn write(&mut self, balance_key: PolkadexBalanceKey, free: u128, reserved: u128) {
        self.general_db
            .write(balance_key.encode(), Balances { free, reserved }.encode());
    }

    pub fn _find(&self, k: PolkadexBalanceKey) -> Result<Balances> {
        println!("Searching for Key");
        match self.general_db._find(k.encode()) {
            Some(v) => Balances::decode(&mut v.as_slice()).map_err(PolkadexDBError::DecodeError),
            None => {
                println!("Key returns None");
                Err(PolkadexDBError::KeyNotFound)
            }
        }
    }

    pub fn _delete(&mut self, k: PolkadexBalanceKey) {
        self.general_db._delete(k.encode());
    }

    pub fn take_disk_snapshot(&mut self) -> Result<Vec<u8>> {
        self.general_db.write_disk_from_memory()
    }

    pub fn load_disk_snapshot(&mut self) -> Result<()> {
        if self.general_db.read_disk_into_memory().is_err() {
            return Err(PolkadexDBError::KeyNotFound);
        }
        Ok(())
    }

    pub fn prepare_for_sending(&self) -> Result<Vec<BalancesData>> {
        self.general_db
            .read_all()
            .into_iter()
            .map(|(left, right)| -> Result<BalancesData> {
                let key = PolkadexBalanceKey::decode(&mut left.as_slice())
                    .map_err(PolkadexDBError::DecodeError)?;
                let balances = Balances::decode(&mut right.as_slice())
                    .map_err(PolkadexDBError::DecodeError)?;
                Ok(BalancesData {
                    asset_id: key.asset_id,
                    account_id: key.account_id,
                    free: balances.free,
                    reserved: balances.reserved,
                })
            })
            .collect()
    }
}

pub fn initialize_balances_mirror() {
    let storage_ptr = Arc::new(Mutex::<BalancesMirror<DiskStorageHandler>>::new(
        BalancesMirror {
            general_db: GeneralDB::new(
                HashMap::new(),
                DiskStorageHandler::open_default(PathBuf::from(BALANCE_DISK_STORAGE_FILENAME)),
            ),
        },
    ));
    let ptr = Arc::into_raw(storage_ptr);
    BALANCES_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
}

pub fn load_balances_mirror() -> Result<&'static Mutex<BalancesMirror<DiskStorageHandler>>> {
    let ptr =
        BALANCES_MIRROR.load(Ordering::SeqCst) as *mut Mutex<BalancesMirror<DiskStorageHandler>>;
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
    use crate::polkadex_db::{Balances, BalancesMirror, PolkadexBalanceKey};
    use codec::Encode;
    use polkadex_primitives::AccountId;
    use polkadex_sgx_primitives::{AssetId, BalancesData};
    use sp_core::{ed25519 as ed25519_core, Pair};
    use std::collections::HashMap;

    fn create_dummy_key() -> PolkadexBalanceKey {
        PolkadexBalanceKey::from(
            AssetId::POLKADEX,
            AccountId::from(
                ed25519_core::Pair::from_seed(b"12345678901234567890123456789012").public(),
            ),
        )
    }
    fn create_secondary_dummy_key() -> PolkadexBalanceKey {
        PolkadexBalanceKey::from(
            AssetId::DOT,
            AccountId::from(
                ed25519_core::Pair::from_seed(b"01234567890123456789012345678901").public(),
            ),
        )
    }

    #[test]
    fn write() {
        let dummy_key = create_dummy_key();
        let mut balances_mirror = BalancesMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        assert_eq!(balances_mirror.general_db.db, HashMap::new());
        balances_mirror.write(dummy_key.clone(), 42u128, 0u128);
        assert_eq!(
            balances_mirror.general_db.db.get(&dummy_key.encode()),
            Some(
                &Balances {
                    free: 42u128,
                    reserved: 0u128
                }
                .encode()
            )
        );
    }

    #[test]
    fn find() {
        let dummy_key = create_dummy_key();
        let mut balances_mirror = BalancesMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        balances_mirror.general_db.db.insert(
            dummy_key.encode(),
            Balances {
                free: 42u128,
                reserved: 0u128,
            }
            .encode(),
        );
        assert_eq!(
            balances_mirror._find(dummy_key).unwrap(),
            Balances {
                free: 42u128,
                reserved: 0u128,
            }
        );
        assert!(balances_mirror._find(create_secondary_dummy_key()).is_err());
    }

    #[test]
    fn delete() {
        let dummy_key = create_dummy_key();
        let mut balances_mirror = BalancesMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        balances_mirror.general_db.db.insert(
            dummy_key.encode(),
            Balances {
                free: 42u128,
                reserved: 0u128,
            }
            .encode(),
        );
        assert!(balances_mirror
            .general_db
            .db
            .contains_key(&dummy_key.encode()));
        balances_mirror._delete(dummy_key.clone());
        assert!(!balances_mirror
            .general_db
            .db
            .contains_key(&dummy_key.encode()));
    }

    #[test]
    fn prepare_for_sending() {
        let dummy_key = create_dummy_key();
        let secondary_dummy_key = create_secondary_dummy_key();
        let mut balances_mirror = BalancesMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        balances_mirror.general_db.db.insert(
            dummy_key.encode(),
            Balances {
                free: 42u128,
                reserved: 0u128,
            }
            .encode(),
        );
        balances_mirror.general_db.db.insert(
            secondary_dummy_key.encode(),
            Balances {
                free: 0u128,
                reserved: 42u128,
            }
            .encode(),
        );
        assert_eq!(
            {
                let mut balances_mirror = balances_mirror
                    .prepare_for_sending()
                    .expect("Unexpected error while preparing to balances nonce data");
                balances_mirror.sort();
                balances_mirror
            },
            vec![
                BalancesData {
                    asset_id: AssetId::POLKADEX,
                    account_id: AccountId::from(
                        ed25519_core::Pair::from_seed(b"12345678901234567890123456789012").public(),
                    ),
                    free: 42u128,
                    reserved: 0u128
                },
                BalancesData {
                    asset_id: AssetId::DOT,
                    account_id: AccountId::from(
                        ed25519_core::Pair::from_seed(b"01234567890123456789012345678901").public(),
                    ),
                    free: 0u128,
                    reserved: 42u128
                }
            ]
        )
    }
}
