use crate::constants::UNIT;
use crate::polkadex::add_main_account;
use crate::polkadex_balance_storage::{
    lock_storage_and_deposit, lock_storage_and_initialize_balance,
};
use crate::polkadex_gateway::lock_storage_get_cache_nonce;
use crate::polkadex_gateway::{process_create_order, settle_trade};
use crate::polkadex_orderbook_storage::lock_storage_and_add_order;
use crate::test_polkadex_gateway::{check_balance, create_mock_gateway};
use polkadex_sgx_primitives::accounts::get_account;
use polkadex_sgx_primitives::types::{
    MarketId, Order, OrderSide, OrderType, OrderUUID, TradeEvent,
};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_tstd::vec::Vec;

pub fn test_happy_path() {
    let gateway = create_mock_gateway();
    let alice: AccountId = get_account("happy_path_user_alice");
    let bob: AccountId = get_account("happy_path_user_bob");
    let token_a = AssetId::BTC;
    let token_b = AssetId::USD;

    // Create Account
    assert!(add_main_account(alice.clone()).is_ok());
    assert!(add_main_account(bob.clone()).is_ok());

    //Initialize Balance
    assert!(lock_storage_and_initialize_balance(alice.clone(), token_a).is_ok());
    assert!(lock_storage_and_initialize_balance(bob.clone(), token_b).is_ok());

    //Deposit some balance
    assert!(lock_storage_and_deposit(alice.clone(), token_a, 500 * UNIT).is_ok());
    assert!(lock_storage_and_deposit(bob.clone(), token_b, 500 * UNIT).is_ok());

    //Check Balance
    assert_eq!(
        check_balance(500 * UNIT, 0u128, alice.clone(), token_a),
        Ok(())
    );

    assert_eq!(
        check_balance(500 * UNIT, 0u128, bob.clone(), token_b),
        Ok(())
    );

    //Place Ask Limit Order
    let mut ask_limit_order: Order = Order {
        user_uid: alice.clone(),
        market_id: MarketId {
            base: token_a,
            quote: token_b,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        quantity: 50 * UNIT,
        price: Some(UNIT),
    };

    let mut buy_limit_order: Order = Order {
        user_uid: bob.clone(),
        market_id: MarketId {
            base: token_a,
            quote: token_b,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 50 * UNIT,
        price: Some(UNIT),
    };

    // Place Ask limit Order
    assert!(gateway
        .place_order(alice.clone(), None, ask_limit_order.clone())
        .is_ok());

    let ask_limit_order_request_id = lock_storage_get_cache_nonce().unwrap() - 1;
    let ask_limit_order_uuid: OrderUUID = (200..202).collect();
    process_create_order(ask_limit_order_request_id, ask_limit_order_uuid.clone());

    // Place Bid Limit Order
    assert!(gateway
        .place_order(bob.clone(), None, buy_limit_order.clone())
        .is_ok());

    let bid_limit_order_request_id = lock_storage_get_cache_nonce().unwrap() - 1;
    let buy_limit_order_uuid: OrderUUID = (202..204).collect();
    process_create_order(bid_limit_order_request_id, buy_limit_order_uuid.clone());

    //Order Event
    let trade_event = TradeEvent {
        market_id: MarketId {
            base: token_a,
            quote: token_b,
        },
        trade_id: 1,
        price: 1 * UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: alice.clone(), // Alice
        maker_order_id: 1,
        maker_order_uuid: ask_limit_order_uuid, //Ask
        taker_user_id: bob.clone(),             // bob
        taker_order_id: 2,
        taker_order_uuid: buy_limit_order_uuid, //Buy
        maker_side: OrderSide::ASK,             //Ask
        timestamp: 23,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(check_balance(450 * UNIT, 0, alice.clone(), token_a), Ok(()));

    assert_eq!(
        check_balance(50 * UNIT, 0u128, alice.clone(), token_b),
        Ok(())
    );

    assert_eq!(
        check_balance(50 * UNIT, 0u128, bob.clone(), token_a),
        Ok(())
    );

    assert_eq!(
        check_balance(450 * UNIT, 0u128, bob.clone(), token_b),
        Ok(())
    );
}
