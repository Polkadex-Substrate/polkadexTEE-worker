use crate::polkadex_orderbook_storage::create_in_memory_orderbook_storage;
use crate::polkadex_orderbook_storage::{load_orderbook, OrderbookStorage};
use polkadex_primitives::types::{Order, OrderSide, OrderType};
use sgx_tstd::string::String;

#[allow(unused)]
pub fn test_create_orderbook_storage() {
    assert_eq!(create_in_memory_orderbook_storage(vec![]).is_ok(), true);
    assert_eq!(load_orderbook().is_ok(), true);
}

#[allow(unused)]
pub fn test_orderbook() {
    let order: Order = Order {
        user_uid: String::from("14dQ6XGcrk4njhYB7ihcjHyyKbFKUVCXt5vffTV9yAWcgrbu").into_bytes(),
        market_id: String::from("btcusd").into_bytes(),
        market_type: String::from("trusted").into_bytes(),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 10,
        price: Some(10000u128),
    };
    let second_order: Order = Order {
        user_uid: String::from("14dQ6XGcrk4njhYB7ihcjHyyKbFKUVCXt5vffTV9yAWcgrbu").into_bytes(),
        market_id: String::from("btcusd").into_bytes(),
        market_type: String::from("trusted").into_bytes(),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 10,
        price: Some(10001u128),
    };

    let mut orderbook: OrderbookStorage = OrderbookStorage::create();

    // Test Add Order
    assert_eq!(
        orderbook
            .add_order("1245-2345-6798-123123".parse().unwrap(), order.clone())
            .is_none(),
        true
    );

    // Test Set Order
    assert_eq!(
        orderbook
            .set_order(
                "1245-2345-6798-123123".parse().unwrap(),
                second_order.clone()
            )
            .is_some(),
        true
    );

    // Test Read Order
    assert_eq!(
        orderbook
            .read_order(&"1245-2345-6798-123123".parse().unwrap())
            .is_some(),
        true
    );

    // Test Remove Order
    assert_eq!(
        orderbook
            .remove_order(&"1245-2345-6798-123123".parse().unwrap())
            .is_some(),
        true
    );

    // Test If Order is removed
    assert_eq!(
        orderbook
            .read_order(&"1245-2345-6798-123123".parse().unwrap())
            .is_none(),
        true
    );
}