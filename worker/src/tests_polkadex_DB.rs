use std::{thread, time};

use polkadex_primitives::types::{Order, OrderSide, OrderType, SignedOrder};

use crate::polkadex_db::{KVStore, RocksDB};

#[test]
fn test_db_initialization() {
    assert!(RocksDB::initialize_db(true).is_ok());
    assert!(RocksDB::load_orderbook_mirror().is_ok());
}

#[test]
fn test_write() {

    // Since Cargo tests run parallel, we need to wait for DB to finish initialization
    thread::sleep(time::Duration::new(2,0));
    let first_order = SignedOrder {
        order_id: "FIRST_ORDER".to_string(),
        order: Order {
            user_uid: "FOO".to_string(),
            market_id: "FLEA_MARKET".to_string(),
            market_type: "SOME_MARKET_TYPE".to_string(),
            order_type: OrderType::LIMIT,
            side: OrderSide::BID,
            quantity: 0,
            price: Some(100u128),
        },
        signature: vec![],
    };

    let handler = RocksDB::write(
        "FIRST_ORDER", first_order.clone());

    let result = handler.join().unwrap();
    assert!(result.is_ok());

    let order_read = RocksDB::find("FIRST_ORDER")
        .unwrap_or(Some(SignedOrder::default()));

    assert!(order_read.is_some());
    assert_eq!(order_read.unwrap(), first_order);

    let second_result = RocksDB::find("SECOND_ORDER");
    assert!(second_result.is_ok());
    assert!(second_result.ok().unwrap().is_none())
}