use polkadex_sgx_primitives::types::{MarketId, Order, OrderSide, OrderType, SignedOrder};
use sgx_tstd::string::String;
use sgx_tstd::sync::SgxMutexGuard;
use sgx_tstd::vec::Vec;
use sgx_tstd::{thread, time};
use sp_core::ed25519::Signature;

use crate::ed25519;
use crate::polkadex_orderbook_storage::create_in_memory_orderbook_storage;
use crate::polkadex_orderbook_storage::{load_orderbook, OrderbookStorage};
use polkadex_sgx_primitives::accounts::get_account;
use polkadex_sgx_primitives::AssetId;

pub fn get_dummy_orders() -> Vec<Order> {
    let order: Order = Order {
        user_uid: get_account("test_account"),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: String::from("trusted").into_bytes(),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 10,
        amount_reserved: 100000u128,
        price: Some(10000u128),
    };
    let second_order: Order = Order {
        user_uid: get_account("test_account"),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: String::from("trusted").into_bytes(),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 10,
        amount_reserved: 100000u128,
        price: Some(10001u128),
    };
    vec![order, second_order]
}

#[allow(unused)]
pub fn test_create_orderbook_storage() {
    let mut signed_orders: Vec<SignedOrder> = vec![];
    let signer_pair = ed25519::unseal_pair().unwrap();
    let mut counter: u8 = 0;
    for order in get_dummy_orders() {
        let mut signed_order = SignedOrder {
            order_id: vec![counter],
            order,
            signature: Signature::default(),
        };
        signed_order.sign(&signer_pair);
        signed_orders.push(signed_order);
        counter += 1;
    }
    assert_eq!(
        create_in_memory_orderbook_storage(signed_orders).is_ok(),
        true
    );
    assert_eq!(load_orderbook().is_ok(), true);
}

#[allow(unused)]
pub fn test_add_orderbook() {
    thread::sleep(time::Duration::new(2, 0));
    let mutex = load_orderbook().unwrap();
    let mut orderbook: SgxMutexGuard<OrderbookStorage> = mutex.lock().unwrap();
    let dummy_orders: Vec<Order> = get_dummy_orders();
    let dummy_orders_count: u8 = get_dummy_orders().len() as u8;
    assert_eq!(
        orderbook
            .add_order(vec![dummy_orders_count + 1 as u8], dummy_orders[0].clone())
            .is_none(),
        true
    );

    let read_order = orderbook.read_order(&vec![dummy_orders_count + 1 as u8]);
    assert_eq!(read_order.is_some(), true);
    assert_eq!(read_order, Some(&dummy_orders[0]));
}

#[allow(unused)]
pub fn test_remove_orderbook() {
    // thread::sleep(time::Duration::new(2, 0));
    let mutex = load_orderbook().unwrap();
    let mut orderbook: SgxMutexGuard<OrderbookStorage> = mutex.lock().unwrap();
    let dummy_orders: Vec<Order> = get_dummy_orders();
    let dummy_orders_count: u8 = get_dummy_orders().len() as u8;
    assert_eq!(
        orderbook
            .add_order(vec![dummy_orders_count + 2 as u8], dummy_orders[0].clone())
            .is_none(),
        true
    );

    let read_order = orderbook.read_order(&vec![dummy_orders_count + 2 as u8]);
    assert_eq!(read_order.is_some(), true);
    assert_eq!(read_order, Some(&dummy_orders[0]));

    let removed_order = orderbook.remove_order(&vec![dummy_orders_count + 2 as u8]);
    assert_eq!(removed_order.is_some(), true);
    assert_eq!(removed_order, Some(dummy_orders[0].clone()));

    let read_order = orderbook.read_order(&vec![dummy_orders_count + 2 as u8]);
    assert_eq!(read_order.is_some(), false);
}

#[allow(unused)]
pub fn test_read_orderbook() {
    // thread::sleep(time::Duration::new(2, 0));
    let mutex = load_orderbook().unwrap();
    let mut orderbook: SgxMutexGuard<OrderbookStorage> = mutex.lock().unwrap();

    for counter in 0..get_dummy_orders().len() as u8 {
        assert_eq!(orderbook.read_order(&vec![counter]).is_some(), true);
    }
    assert_eq!(
        orderbook
            .read_order(&vec![get_dummy_orders().len() as u8])
            .is_some(),
        false
    );
}
