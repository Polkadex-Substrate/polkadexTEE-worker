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
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use crate::polkadex_db::{GeneralDB, PolkadexDBError};
use polkadex_sgx_primitives::{AccountId, AssetId};

static BALANCES_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[derive(Debug)]
pub struct BalancesMirror {
    general_db: GeneralDB,
}

#[derive(Encode, Decode)]
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

impl BalancesMirror {
    pub fn write(&mut self, balance_key: PolkadexBalanceKey, free: u128, reserved: u128) {
        self.general_db
            .write(balance_key.encode(), Balances { free, reserved }.encode());
    }

    pub fn _find(&self, k: PolkadexBalanceKey) -> Result<Balances, PolkadexDBError> {
        println!("Searching for Key");
        match self.general_db._find(k.encode()) {
            Some(v) => Ok(Balances::decode(&mut v.as_slice()).unwrap()),
            None => {
                println!("Key returns None");
                Err(PolkadexDBError::_KeyNotFound)
            }
        }
    }

    pub fn _delete(&mut self, k: PolkadexBalanceKey) {
        self.general_db._delete(k.encode());
    }
}

pub fn initialize_balances_mirror() {
    let storage_ptr = Arc::new(Mutex::<BalancesMirror>::new(BalancesMirror {
        general_db: GeneralDB { db: HashMap::new() },
    }));
    let ptr = Arc::into_raw(storage_ptr);
    BALANCES_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
}

pub fn load_balances_mirror() -> Result<&'static Mutex<BalancesMirror>, PolkadexDBError> {
    let ptr = BALANCES_MIRROR.load(Ordering::SeqCst) as *mut Mutex<BalancesMirror>;
    if ptr.is_null() {
        println!("Unable to load the pointer");
        Err(PolkadexDBError::UnableToLoadPointer)
    } else {
        Ok(unsafe { &*ptr })
    }
}
