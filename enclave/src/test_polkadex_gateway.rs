use log::*;
use polkadex_sgx_primitives::accounts::get_account;
use polkadex_sgx_primitives::types::{
    MarketId, Order, OrderSide, OrderType, OrderUUID, TradeEvent,
};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_tstd::vec::Vec;

// Polkadex represents 1 Token as 10^^18 minimum possible units
use crate::constants::UNIT;
use crate::polkadex::{add_main_account, create_in_memory_account_storage};
use crate::polkadex_balance_storage::{
    create_in_memory_balance_storage, lock_storage_and_deposit, lock_storage_and_get_balances,
    lock_storage_and_initialize_balance,
};
use crate::polkadex_gateway::{authenticate_user, cancel_order, place_order, settle_trade};
use crate::polkadex_orderbook_storage::{
    create_in_memory_orderbook_storage, lock_storage_and_add_order, lock_storage_and_get_order,
};
use crate::test_proxy::initialize_dummy;

pub fn initialize_storage() {
    // Initialize Gateway
    // initialize_polkadex_gateway();
    // Initialize Account Storage
    assert!(create_in_memory_account_storage(vec![]).is_ok());
    // Initialize Balance storage
    assert!(create_in_memory_balance_storage().is_ok());
    // Initialize Order Mirror
    assert!(create_in_memory_orderbook_storage(vec![]).is_ok());
}

fn setup(main: AccountId) {
    // Register Main account
    assert!(add_main_account(main.clone()).is_ok());
    // Initialize Balance for main account
    assert!(lock_storage_and_initialize_balance(main.clone(), AssetId::POLKADEX).is_ok());
    assert!(lock_storage_and_initialize_balance(main.clone(), AssetId::DOT).is_ok());
    // Deposit some balance
    assert!(lock_storage_and_deposit(main.clone(), AssetId::POLKADEX, 100 * UNIT).is_ok());
    assert!(lock_storage_and_deposit(main.clone(), AssetId::DOT, 100 * UNIT).is_ok());
}

fn check_balance(free: u128, reserved: u128, main: AccountId, token: AssetId) -> Result<(), u32> {
    let balance = lock_storage_and_get_balances(main, token).unwrap();
    if balance.free != free {
        error!("Expected Free balance: {}, Given: {}", balance.free, free);
        return Err(0);
    }
    if balance.reserved != reserved {
        error!(
            "Expected Reserved balance: {}, Given: {}",
            balance.reserved, reserved
        );
        return Err(1);
    }
    Ok(())
}

pub fn test_place_limit_buy_order() {
    let main: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: UNIT,
        amount_reserved: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::DOT).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(place_order(main.clone(), None, new_order.clone()).is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    new_order.price = Some(99 * UNIT);
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(0u128, 100 * UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap();
}

pub fn test_place_limit_sell_order() {
    let main: AccountId = get_account("test_place_limit_sell_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        quantity: UNIT,
        amount_reserved: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(place_order(main.clone(), None, new_order.clone()).is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    new_order.price = Some(99 * UNIT);
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(98 * UNIT, 2 * UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::DOT).unwrap();
}

pub fn test_place_market_buy_order() {
    let main: AccountId = get_account("test_place_market_buy_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        quantity: UNIT,
        amount_reserved: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::DOT).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.price = Some(100 * UNIT);
    assert!(place_order(main.clone(), None, new_order.clone()).is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.price = Some(99 * UNIT);
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(0u128, 100 * UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap();
}

pub fn test_place_market_sell_order() {
    let main: AccountId = get_account("test_place_market_sell_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::ASK,
        quantity: UNIT,
        amount_reserved: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(place_order(main.clone(), None, new_order.clone()).is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(98 * UNIT, 2 * UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::DOT).unwrap();
}

pub fn test_cancel_limit_buy_order() {}

pub fn test_cancel_limit_sell_order() {}

pub fn test_authenticate_user() {
    initialize_dummy();
    let main: AccountId = get_account("first_account");
    let proxy: AccountId = get_account("first_dummy_account");
    // Without Proxy
    assert!(authenticate_user(main.clone(), None).is_ok());
    // With Proxy
    assert!(authenticate_user(main.clone(), Some(proxy)).is_ok());
    let not_main: AccountId = get_account("not_registered_main");
    let not_proxy: AccountId = get_account("not_registered_proxy");
    // Should error since not_main is not registered
    assert!(authenticate_user(not_main.clone(), None).is_err());
    // Should error since not_proxy is not registered under main.
    assert!(authenticate_user(main, Some(not_proxy.clone())).is_err());
    // Should error since both proxy and main is not registered
    assert!(authenticate_user(not_main, Some(not_proxy)).is_err());
}

// ALL ASK LIMIT TEST

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_ask_limit() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: UNIT,
        amount_reserved: 2 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order"); //Alice
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        quantity: UNIT,
        amount_reserved: UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    );
    // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_full_ask_limit() {
    setup_place_buy_and_sell_order_full_ask_limit();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: 1 * UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: buy_order_uuid,
        taker_order_id: vec![],
        taker_order_uuid: sell_order_uuid,
        maker_side: OrderSide::BID,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(99 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_ask_limit() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 4 * UNIT,
        quantity: 2 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(96 * UNIT, 4 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        amount_reserved: UNIT,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}
#[allow(unused)]
pub fn test_settle_trade_partial_ask_limit() {
    setup_place_buy_and_sell_order_partial_ask_limit();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: buy_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: sell_order_uuid,
        maker_side: OrderSide::BID,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(96 * UNIT, 3 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(buy_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 3 * UNIT,
        quantity: 1 * UNIT,
        price: Some(2 * UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_ask_limit() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial_two");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 2 * UNIT,
        quantity: 1 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    error!("tesm1");
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    error!("temp2");
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial_two");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        amount_reserved: 2 * UNIT,
        quantity: 2 * UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    error!("temp3");
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    error!("temp4");
    assert_eq!(
        check_balance(
            98 * UNIT,
            2 * UNIT,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}
#[allow(unused)]
pub fn test_settle_trade_partial_two_ask_limit() {
    setup_place_buy_and_sell_order_partial_two_ask_limit();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial_two");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial_two");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: 1 * UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: buy_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: sell_order_uuid.clone(),
        maker_side: OrderSide::BID,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(
            98 * UNIT,
            1 * UNIT,
            sell_order_user.clone(),
            AssetId::POLKADEX
        ),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(99 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        amount_reserved: 1 * UNIT,
        side: OrderSide::ASK,
        quantity: 1 * UNIT,
        price: Some(UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

// ALL BUY LIMIT TEST

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_buy_limit() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 2 * UNIT,
        quantity: UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        amount_reserved: UNIT,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_full_buy_limit() {
    setup_place_buy_and_sell_order_full_buy_limit();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: sell_order_uuid,
        taker_order_id: vec![],
        taker_order_uuid: buy_order_uuid,
        maker_side: OrderSide::ASK,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(99 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_buy_limit() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 4 * UNIT,
        quantity: 2 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(96 * UNIT, 4 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        amount_reserved: UNIT,
        side: OrderSide::ASK,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    );
    // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_partial_buy_limit() {
    setup_place_buy_and_sell_order_partial_buy_limit();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: sell_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: buy_order_uuid.clone(),
        maker_side: OrderSide::ASK,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    ); //101
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(96 * UNIT, 3 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(buy_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 3 * UNIT,
        quantity: 1 * UNIT,
        price: Some(2 * UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_buy_limit() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial_two");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 2 * UNIT,
        quantity: 1 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    error!("tesm1");
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    error!("temp2");
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial_two");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        amount_reserved: 2 * UNIT,
        quantity: 2 * UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    error!("temp3");
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    error!("temp4");
    assert_eq!(
        check_balance(
            98 * UNIT,
            2 * UNIT,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_partial_two_buy_limit() {
    setup_place_buy_and_sell_order_partial_two_buy_limit();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial_two");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial_two");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: sell_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: buy_order_uuid.clone(),
        maker_side: OrderSide::ASK,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(
            98 * UNIT,
            1 * UNIT,
            sell_order_user.clone(),
            AssetId::POLKADEX
        ),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(99 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        amount_reserved: 1 * UNIT,
        quantity: 1 * UNIT,
        price: Some(UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}
//ALL SELL MARKET ORDER

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_ask_market() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: UNIT,
        amount_reserved: 2 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let main: AccountId = get_account("test_place_market_sell_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        amount_reserved: UNIT,
        side: OrderSide::ASK,
        quantity: UNIT,
        price: None,
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_full_ask_market() {
    setup_place_buy_and_sell_order_full_ask_market();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_market_sell_order");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: 1 * UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: buy_order_uuid,
        taker_order_id: vec![],
        taker_order_uuid: sell_order_uuid,
        maker_side: OrderSide::BID,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_ask_market() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 2 * UNIT,
        amount_reserved: 4 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(96 * UNIT, 4 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let main: AccountId = get_account("test_place_market_sell_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        amount_reserved: UNIT,
        side: OrderSide::ASK,
        quantity: UNIT,
        price: None,
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_partial_ask_market() {
    setup_place_buy_and_sell_order_partial_ask_market();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_market_sell_order");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: 2 * UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: buy_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: sell_order_uuid,
        maker_side: OrderSide::BID,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(96 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(buy_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        amount_reserved: 2 * UNIT,
        quantity: 1 * UNIT,
        price: Some(2 * UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_ask_market() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 1 * UNIT,
        amount_reserved: 2 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let main: AccountId = get_account("test_place_market_sell_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        amount_reserved: UNIT,
        side: OrderSide::ASK,
        quantity: 2 * UNIT,
        price: None,
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(98 * UNIT, 2 * UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_partial_two_ask_market() {
    setup_place_buy_and_sell_order_partial_two_ask_market();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_market_sell_order");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: 1 * UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: buy_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: sell_order_uuid.clone(),
        maker_side: OrderSide::BID,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(
            98 * UNIT,
            1 * UNIT,
            sell_order_user.clone(),
            AssetId::POLKADEX
        ),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        amount_reserved: 1 * UNIT,
        side: OrderSide::ASK,
        quantity: 1 * UNIT,
        price: None,
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

//ALL BUY MARKET ORDER

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_buy_market() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        amount_reserved: 2 * UNIT,
        quantity: UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        amount_reserved: UNIT,
        quantity: 2 * UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(
            98 * UNIT,
            2 * UNIT,
            sell_order_user.clone(),
            AssetId::POLKADEX
        ),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_full_buy_market() {
    setup_place_buy_and_sell_order_full_buy_market();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: sell_order_uuid,
        taker_order_id: vec![],
        taker_order_uuid: buy_order_uuid,
        maker_side: OrderSide::ASK,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(98 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_bid_market() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        amount_reserved: 2 * UNIT,
        quantity: 2 * UNIT, //Remove later
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        amount_reserved: UNIT,
        side: OrderSide::ASK,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    );
    // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_partial_bid_market() {
    setup_place_buy_and_sell_order_partial_bid_market();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: sell_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: buy_order_uuid.clone(),
        maker_side: OrderSide::ASK,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    ); //101
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 1 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(buy_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        amount_reserved: 1 * UNIT,
        quantity: 2 * UNIT,
        price: Some(1 * UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_bid_market() {
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        amount_reserved: 2 * UNIT,
        quantity: 2 * UNIT, //Remove later
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(place_order(buy_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());

    // SELL LIMIT ORDER
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let mut new_order: Order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        amount_reserved: 3 * UNIT,
        side: OrderSide::ASK,
        quantity: 3 * UNIT,
        price: Some(UNIT),
    };

    setup(sell_order_user.clone());
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    );
    // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(sell_order_user.clone(), None, new_order.clone()).is_ok());
    assert_eq!(
        check_balance(
            97 * UNIT,
            3 * UNIT,
            sell_order_user.clone(),
            AssetId::POLKADEX
        ),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..202).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

#[allow(unused)]
pub fn test_settle_trade_partial_two_bid_market() {
    setup_place_buy_and_sell_order_partial_two_bid_market();
    let sell_order_uuid: OrderUUID = (200..202).collect();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        trade_id: vec![],
        price: 0,
        amount: UNIT,
        funds: 0,
        maker_order_id: vec![],
        maker_order_uuid: sell_order_uuid.clone(),
        taker_order_id: vec![],
        taker_order_uuid: buy_order_uuid.clone(),
        maker_side: OrderSide::ASK,
        timestamp: vec![],
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(97 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user.clone(), AssetId::DOT),
        Ok(())
    ); //101
    assert_eq!(
        check_balance(102 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        amount_reserved: 1 * UNIT,
        quantity: 1 * UNIT,
        price: Some(1 * UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}
