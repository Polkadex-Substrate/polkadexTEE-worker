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
use polkadex_sgx_primitives::AccountId;

static BALANCES_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub struct BalancesMirror {
    general_db: GeneralDB,
}

#[derive(Encode, Decode)]
pub struct Balances {
    free: u128,
    reserved: u128,
}

impl BalancesMirror {
    pub fn write(&mut self, account_id: AccountId, free: u128, reserved: u128) {
        self.general_db
            .write(account_id.encode(), Balances { free, reserved }.encode());
    }

    pub fn _find(&self, k: AccountId) -> Result<Balances, PolkadexDBError> {
        println!("Searching for Key");
        match self.general_db._find(k.encode()) {
            Some(v) => Ok(Balances::decode(&mut v.as_slice()).unwrap()),
            None => {
                println!("Key returns None");
                Err(PolkadexDBError::_KeyNotFound)
            }
        }
    }

    pub fn _delete(&mut self, k: AccountId) {
        self.general_db._delete(k.encode());
    }

    // pub fn read_all(&self) -> Result<Vec<u32>, PolkadexDBError> {
    //     let iterator = self.general_db.read_all().into_iter();
    //     let mut nonces: Vec<u32> = vec![];
    //     for (_, value) in iterator.take(ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT) {
    //         match SignedOrder::from_vec(&*value) {
    //             Ok(order) => orders.push(order),
    //             Err(_) => {
    //                 println!("Unable to deserialize");
    //                 return Err(PolkadexDBError::UnableToDeseralizeValue);
    //             }
    //         }
    //     }
    //     Ok(orders)
    // }
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
