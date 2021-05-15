use polkadex_primitives::types::{Order, OrderSide, OrderType};
use sgx_tstd::string::String;
use sgx_tstd::vec::Vec;

use crate::polkadex_orderbook_storage::{load_orderbook, OrderbookStorage};
use crate::polkadex_orderbook_storage::create_in_memory_orderbook_storage;

#[allow(unused)]
pub fn test_create_orderbook_storage() {
    assert_eq!(create_in_memory_orderbook_storage(vec![]).is_ok(), true);
    assert_eq!(load_orderbook().is_ok(), true);
}

pub fn get_dummy_orders() -> Vec<Order> {
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
    vec![order, second_order]
}

#[allow(unused)]
pub fn test_orderbook() {

}
// #[allow(unused)]
// pub fn test_read_orderbook() {
//     let dummy_orders = get_dummy_orders();
//     let order = dummy_orders[0];
//     let second_order = dummy_orders[1];
//     // Test Add Order
//     assert_eq!(
//         orderbook
//             .add_order("1245-2345-6798-123123".parse().unwrap(), order.clone())
//             .is_none(),
//         true
//     );
//
//     // Test Set Order
//     assert_eq!(
//         orderbook
//             .set_order(
//                 "1245-2345-6798-123123".parse().unwrap(),
//                 second_order.clone(),
//             )
//             .is_some(),
//         true
//     );
//
//     // Test Read Order
//     assert_eq!(
//         orderbook
//             .read_order(&"1245-2345-6798-123123".parse().unwrap())
//             .is_some(),
//         true
//     );
//
//     // Test Remove Order
//     assert_eq!(
//         orderbook
//             .remove_order(&"1245-2345-6798-123123".parse().unwrap())
//             .is_some(),
//         true
//     );
//
//     // Test If Order is removed
//     assert_eq!(
//         orderbook
//             .read_order(&"1245-2345-6798-123123".parse().unwrap())
//             .is_none(),
//         true
//     );
// }