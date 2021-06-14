use std::{thread, time};

use polkadex_sgx_primitives::types::{Order, OrderSide, OrderType, SignedOrder, MarketId};
use polkadex_sgx_primitives::accounts::get_account;
use crate::polkadex_db::{KVStore, PolkadexDBError, RocksDB};
use sp_core::ed25519::Signature;
use std::sync::MutexGuard;
use polkadex_sgx_primitives::AssetId;

#[test]
fn test_db_initialization() {
    assert!(RocksDB::initialize_db(true).is_ok());
    assert!(RocksDB::load_orderbook_mirror().is_ok());
}

#[test]
fn test_write_and_delete() {
    // Since Cargo tests run parallel, we need to wait for DB to finish initialization
    thread::sleep(time::Duration::new(2, 0));
    let first_order = SignedOrder {
        order_id: "FIRST_ORDER".to_string().into_bytes(),
        order: Order {
            user_uid: get_account("FOO"),
            market_id: MarketId{
                base: AssetId::POLKADEX,
                quote: AssetId::DOT
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
        let mutex = RocksDB::load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
        RocksDB::write(
            &orderbook_mirror,
            "FIRST_ORDER".to_string().into_bytes(),
            &first_order,
        )
    });

    let result = handler.join().unwrap();
    assert!(result.is_ok());

    let order_read = RocksDB::find("FIRST_ORDER".to_string().into_bytes())
        .unwrap_or(Some(SignedOrder::default()));

    assert!(order_read.is_some());
    assert_eq!(order_read.unwrap(), first_order_clone);

    let second_result = RocksDB::find("SECOND_ORDER".to_string().into_bytes());
    assert!(second_result.is_ok());
    assert!(second_result.ok().unwrap().is_none());

    let delete_handler = thread::spawn(move || -> Result<(), PolkadexDBError> {
        let mutex = RocksDB::load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
        RocksDB::delete(&orderbook_mirror, "FIRST_ORDER".to_string().into_bytes())
    });

    let result = delete_handler.join().unwrap();
    assert!(result.is_ok());

    let second_result = RocksDB::find("FIRST_ORDER".to_string().into_bytes());
    assert!(second_result.is_ok());
    assert!(second_result.ok().unwrap().is_none());
}

#[test]
fn test_read_all() {
    // Since Cargo tests run parallel, we need to wait for DB to finish initialization
    thread::sleep(time::Duration::new(2, 0));
    let first_order = SignedOrder {
        order_id: "FIRST_ORDER1".to_string().into_bytes(),
        order: Order {
            user_uid: get_account("FOO"),
            market_id: MarketId{
                base: AssetId::POLKADEX,
                quote: AssetId::DOT
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
            market_id: MarketId{
                base: AssetId::POLKADEX,
                quote: AssetId::DOT
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
            user_uid:get_account("FOO"),
            market_id: MarketId{
                base: AssetId::POLKADEX,
                quote: AssetId::DOT
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
        let mutex = RocksDB::load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
        RocksDB::write(
            &orderbook_mirror,
            "FIRST_ORDER1".to_string().into_bytes(),
            &first_order,
        )
    });

    let result = handler.join().unwrap();
    assert!(result.is_ok());

    let handler = thread::spawn(move || -> Result<(), PolkadexDBError> {
        let mutex = RocksDB::load_orderbook_mirror()?;
        let orderbook_mirror: MutexGuard<RocksDB> = mutex.lock().unwrap();
        RocksDB::write(
            &orderbook_mirror,
            "SECOND_ORDER1".to_string().into_bytes(),
            &second_order,
        )
    });

    let result = handler.join().unwrap();
    assert!(result.is_ok());

    let orders = RocksDB::read_all().ok().unwrap();
    assert!(orders.contains(&first_order_clone));
    assert!(orders.contains(&second_order_clone));
    assert!(!orders.contains(&third_order));
}
