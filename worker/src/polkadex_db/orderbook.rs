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

use super::Result;
use crate::constants::{ORDERBOOK_DISK_STORAGE_FILENAME, ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT};
use codec::Encode;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use crate::polkadex_db::{GeneralDB, PolkadexDBError};
use polkadex_sgx_primitives::types::SignedOrder;

use super::disk_storage_handler::DiskStorageHandler;
use super::PermanentStorageHandler;
use polkadex_sgx_primitives::OrderbookData;

static ORDERBOOK_MIRROR: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[derive(Debug)]
pub struct OrderbookMirror<D: PermanentStorageHandler> {
    pub general_db: GeneralDB<D>,
}

impl<D: PermanentStorageHandler> OrderbookMirror<D> {
    pub fn write(&mut self, order_uid: Vec<u8>, signed_order: &SignedOrder) {
        self.general_db.write(order_uid, signed_order.encode());
    }

    pub fn _find(&self, k: Vec<u8>) -> Result<SignedOrder> {
        println!("Searching for Key");
        match self.general_db._find(k) {
            Some(v) => match SignedOrder::from_vec(&v) {
                Ok(order) => {
                    println!("Found Key");
                    Ok(order)
                }
                Err(_) => {
                    println!("Unable to Deserialize");
                    Err(PolkadexDBError::UnableToDeseralizeValue)
                }
            },
            None => {
                println!("Key returns None");
                Err(PolkadexDBError::_KeyNotFound)
            }
        }
    }

    pub fn _delete(&mut self, k: Vec<u8>) {
        self.general_db._delete(k);
    }

    pub fn read_all(&self) -> Result<Vec<SignedOrder>> {
        let iterator = self.general_db.read_all().into_iter();
        let mut orders: Vec<SignedOrder> = vec![];
        for (_, value) in iterator.take(ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT) {
            match SignedOrder::from_vec(&*value) {
                Ok(order) => orders.push(order),
                Err(_) => {
                    println!("Unable to deserialize");
                    return Err(PolkadexDBError::UnableToDeseralizeValue);
                }
            }
        }
        Ok(orders)
    }

    pub fn prepare_for_sending(&self) -> Result<Vec<OrderbookData>> {
        Ok(self
            .read_all()?
            .into_iter()
            .map(|signed_order| OrderbookData { signed_order })
            .collect())
    }

    pub fn take_disk_snapshot(&mut self) -> Result<()> {
        self.general_db.write_disk_from_memory()
    }

    pub fn load_disk_snapshot(&mut self) -> Result<()> {
        if self.general_db.read_disk_into_memory().is_err() {
            return Err(PolkadexDBError::_KeyNotFound);
        }
        Ok(())
    }
}

pub fn initialize_orderbook_mirror() {
    let storage_ptr = Arc::new(Mutex::<OrderbookMirror<DiskStorageHandler>>::new(
        OrderbookMirror {
            general_db: GeneralDB::new(
                HashMap::new(),
                DiskStorageHandler::open_default(PathBuf::from(ORDERBOOK_DISK_STORAGE_FILENAME)),
            ),
        },
    ));
    let ptr = Arc::into_raw(storage_ptr);
    ORDERBOOK_MIRROR.store(ptr as *mut (), Ordering::SeqCst);
}

pub fn load_orderbook_mirror() -> Result<&'static Mutex<OrderbookMirror<DiskStorageHandler>>> {
    let ptr =
        ORDERBOOK_MIRROR.load(Ordering::SeqCst) as *mut Mutex<OrderbookMirror<DiskStorageHandler>>;
    if ptr.is_null() {
        println!("Unable to load the pointer");
        Err(PolkadexDBError::UnableToLoadPointer)
    } else {
        Ok(unsafe { &*ptr })
    }
}

#[cfg(test)]
mod tests {
    use super::{GeneralDB, OrderbookMirror};
    use crate::polkadex_db::mock::PermanentStorageMock;
    use codec::Encode;
    use polkadex_sgx_primitives::types::{MarketId, Order, OrderSide, OrderType, SignedOrder};
    use polkadex_sgx_primitives::AssetId;
    use sp_core::ed25519::Signature;
    use std::collections::HashMap;
    use substratee_worker_primitives::get_account;

    fn first_order() -> SignedOrder {
        SignedOrder {
            order_id: "FIRST_ORDER".to_string().into_bytes(),
            order: Order {
                user_uid: get_account("FOO"),
                market_id: MarketId {
                    base: AssetId::POLKADEX,
                    quote: AssetId::DOT,
                },
                market_type: "SOME_MARKET_TYPE".to_string().into_bytes(),
                order_type: OrderType::LIMIT,
                side: OrderSide::BID,
                quantity: 0,
                price: Some(100u128),
            },
            signature: Signature::default(),
        }
    }

    fn second_order() -> SignedOrder {
        SignedOrder {
            order_id: "SECOND_ORDER".to_string().into_bytes(),
            order: Order {
                user_uid: get_account("BAR"),
                market_id: MarketId {
                    base: AssetId::DOT,
                    quote: AssetId::POLKADEX,
                },
                market_type: "NONE_MARKET_TYPE".to_string().into_bytes(),
                order_type: OrderType::MARKET,
                side: OrderSide::BID,
                quantity: 1,
                price: Some(200u128),
            },
            signature: Signature::default(),
        }
    }

    #[test]
    fn write() {
        let mut orderbook = OrderbookMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        assert_eq!(orderbook.general_db.db, HashMap::new());
        orderbook.write("FIRST_ORDER".encode(), &first_order());
        assert_eq!(
            orderbook.general_db.db.get(&"FIRST_ORDER".encode()),
            Some(&first_order().encode())
        );
    }

    #[test]
    fn find() {
        let mut orderbook = OrderbookMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        orderbook
            .general_db
            .db
            .insert("FIRST_ORDER".encode(), first_order().encode());
        assert_eq!(
            orderbook._find("FIRST_ORDER".encode()).unwrap(),
            first_order()
        );
        assert!(orderbook._find("SECOND_ORDER".encode()).is_err());
    }

    #[test]
    fn delete() {
        let mut orderbook = OrderbookMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        orderbook
            .general_db
            .db
            .insert("FIRST_ORDER".encode(), first_order().encode());
        assert!(orderbook
            .general_db
            .db
            .contains_key(&"FIRST_ORDER".encode()));
        orderbook._delete("FIRST_ORDER".encode());
        assert!(!orderbook
            .general_db
            .db
            .contains_key(&"FIRST_ORDER".encode()));
    }

    #[test]
    fn read_all() {
        let mut orderbook = OrderbookMirror {
            general_db: GeneralDB::new(HashMap::new(), PermanentStorageMock::default()),
        };
        orderbook
            .general_db
            .db
            .insert("FIRST_ORDER".encode(), first_order().encode());
        orderbook
            .general_db
            .db
            .insert("SECOND_ORDER".encode(), second_order().encode());
        assert_eq!(
            {
                let mut read_all = orderbook.read_all().unwrap();
                let mut encoded: Vec<Vec<u8>> = read_all
                    .into_iter()
                    .map(|signed_order| signed_order.encode())
                    .collect();
                encoded.sort();
                read_all = encoded
                    .into_iter()
                    .map(|bytes| SignedOrder::from_vec(&*bytes).unwrap())
                    .collect();
                read_all
            },
            vec![first_order(), second_order()]
        );
    }
}
