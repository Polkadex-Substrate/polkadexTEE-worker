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

use std::{thread, time};

use crate::polkadex_db::{
    orderbook::initialize_orderbook_mirror, orderbook::load_orderbook_mirror, DiskStorageHandler,
    OrderbookMirror, PolkadexDBError,
};
use polkadex_sgx_primitives::types::{MarketId, Order, OrderSide, OrderType, SignedOrder};
use polkadex_sgx_primitives::AssetId;
use sp_core::ed25519::Signature;
use std::sync::MutexGuard;
use substratee_worker_primitives::get_account;

#[test]
fn test_db_initialization() {
    initialize_orderbook_mirror();
    assert!(load_orderbook_mirror().is_ok());
}

#[test]
fn test_write_and_delete() {
    // Since Cargo tests run parallel, we need to wait for DB to finish initialization
    thread::sleep(time::Duration::new(2, 0));
    let first_order = SignedOrder {
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
    };
    let first_order_clone = first_order.clone();

    let handler = thread::spawn(move || -> Result<(), PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror.write("FIRST_ORDER".to_string().into_bytes(), &first_order);
        Ok(())
    });

    let result = handler.join().unwrap();
    assert!(result.is_ok());

    let handler = thread::spawn(move || -> Result<SignedOrder, PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        let signed_order = orderbook_mirror
            ._find("FIRST_ORDER".to_string().into_bytes())
            .unwrap_or_default();
        Ok(signed_order)
    });

    let order_read = handler.join().unwrap().unwrap();

    assert_eq!(order_read, first_order_clone);

    let handler = thread::spawn(move || -> Result<SignedOrder, PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror._find("SECOND_ORDER".to_string().into_bytes())
    });

    let second_result = handler.join().unwrap();

    assert!(second_result.is_err());

    let delete_handler = thread::spawn(move || -> Result<(), PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror._delete("FIRST_ORDER".to_string().into_bytes());
        Ok(())
    });

    let result = delete_handler.join().unwrap();
    assert!(result.is_ok());

    let handler = thread::spawn(move || -> Result<SignedOrder, PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror._find("FIRST_ORDER".to_string().into_bytes())
    });

    let second_result = handler.join().unwrap();

    assert!(second_result.is_err());
}

#[test]
fn test_read_all() {
    // Since Cargo tests run parallel, we need to wait for DB to finish initialization
    thread::sleep(time::Duration::new(2, 0));
    let first_order = SignedOrder {
        order_id: "FIRST_ORDER1".to_string().into_bytes(),
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
    };
    let second_order = SignedOrder {
        order_id: "SECOND_ORDER1".to_string().into_bytes(),
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
    };
    let third_order = SignedOrder {
        order_id: "THIRD_ORDER".to_string().into_bytes(),
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
    };

    let first_order_clone = first_order.clone();
    let second_order_clone = second_order.clone();

    let handler = thread::spawn(move || -> Result<(), PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror.write("FIRST_ORDER1".to_string().into_bytes(), &first_order);
        Ok(())
    });

    let result = handler.join().unwrap();
    assert!(result.is_ok());

    let handler = thread::spawn(move || -> Result<(), PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror.write("SECOND_ORDER1".to_string().into_bytes(), &second_order);
        Ok(())
    });

    let result = handler.join().unwrap();
    assert!(result.is_ok());

    let handler = thread::spawn(move || -> Result<Vec<SignedOrder>, PolkadexDBError> {
        let mutex = load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror.read_all()
    });

    let orders = handler.join().unwrap().unwrap();
    assert!(orders.contains(&first_order_clone));
    assert!(orders.contains(&second_order_clone));
    assert!(!orders.contains(&third_order));
}
