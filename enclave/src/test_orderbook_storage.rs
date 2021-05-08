use crate::polkadex_balance_storage::create_in_memory_balance_storage;
use crate::polkadex_orderbook_storage::{load_balance_storage, load_orderbook, OrderbookStorage};
use openfinex::types::{Order, OrderSide, OrderType};

#[allow(unused)]
pub fn test_create_balance_storage() {
    assert_eq!(create_in_memory_balance_storage().is_ok(), true);
    assert_eq!(load_orderbook().is_ok(), true);
}

#[allow(unused)]
pub fn test_orderbook() {
    let order: Order = Order {
        user_uid: "14dQ6XGcrk4njhYB7ihcjHyyKbFKUVCXt5vffTV9yAWcgrbu"
            .parse()
            .unwrap(),
        market_id: "btcusd".parse().unwrap(),
        market_type: "trusted".parse().unwrap(),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 10,
        price: Some(10000u128),
    };
    let second_order: Order = Order {
        user_uid: "14dQ6XGcrk4njhYB7ihcjHyyKbFKUVCXt5vffTV9yAWcgrbu"
            .parse()
            .unwrap(),
        market_id: "btcusd".parse().unwrap(),
        market_type: "trusted".parse().unwrap(),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 10,
        price: Some(10001u128),
    };

    let mut orderbook: OrderbookStorage = OrderbookStorage::create();

    // Test Add Order
    assert_eq!(
        orderbook.add_order("1245-2345-6798-123123".parse().unwrap(), order.clone()),
        None
    );

    // Test Set Order
    assert_eq!(
        orderbook.set_order(
            "1245-2345-6798-123123".parse().unwrap(),
            second_order.clone()
        ),
        Some(order.clone())
    );

    // Test Read Order
    assert_eq!(
        orderbook.read_order("1245-2345-6798-123123".parse().unwrap()),
        Some(second_order.clone())
    );

    // Test Remove Order
    assert_eq!(
        orderbook.remove_order("1245-2345-6798-123123".parse().unwrap()),
        Some(second_order.clone())
    );

    // Test If Order is removed
    assert_eq!(
        orderbook.read_order("1245-2345-6798-123123".parse().unwrap()),
        None
    );
}
