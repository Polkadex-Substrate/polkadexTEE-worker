use log::*;
use polkadex_sgx_primitives::accounts::get_account;
use polkadex_sgx_primitives::types::{MarketId, Order, OrderSide, OrderType, OrderUUID};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_tstd::vec::Vec;

// Polkadex represents 1 Token as 10^^18 minimum possible units
use crate::constants::UNIT;
use crate::polkadex::{add_main_account, create_in_memory_account_storage};
use crate::polkadex_balance_storage::{
    create_in_memory_balance_storage, lock_storage_and_deposit, lock_storage_and_get_balances,
    lock_storage_and_initialize_balance,
};
use crate::polkadex_gateway::{
    authenticate_user, cancel_order, initialize_polkadex_gateway,
    load_storage_check_nonce_in_insert_order_cache, load_storage_insert_order_cache, place_order,
    process_create_order,
};
use crate::polkadex_orderbook_storage::{
    create_in_memory_orderbook_storage, lock_storage_and_add_order,
    lock_storage_and_check_order_in_orderbook,
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
    initialize_polkadex_gateway();
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

pub fn setup_test_cancel_limit_bid_order() {
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
}

pub fn test_cancel_limit_bid_order() {
    setup_test_cancel_limit_bid_order();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    assert_eq!(
        cancel_order(buy_order_user.clone(), None, buy_order_uuid),
        Ok(())
    );
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );
}

pub fn setup_test_cancel_ask_order() {
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
    let sell_order_uuid: OrderUUID = (200..201).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

pub fn test_cancel_ask_order() {
    setup_test_cancel_ask_order();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let sell_order_uuid: OrderUUID = (200..201).collect();
    assert_eq!(
        cancel_order(sell_order_user.clone(), None, sell_order_uuid),
        Ok(())
    );
    assert_eq!(
        check_balance(
            100 * UNIT,
            0u128,
            sell_order_user.clone(),
            AssetId::POLKADEX,
        ),
        Ok(())
    );
}

pub fn setup_process_create_order() {
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
    let nonce: u128 = 1;
    load_storage_insert_order_cache(nonce, new_order);
    assert_eq!(
        load_storage_check_nonce_in_insert_order_cache(nonce),
        Ok(true)
    );
}

pub fn test_process_create_order() {
    setup_process_create_order();
    let nonce: u128 = 1;
    let order_uuid: OrderUUID = (200..201).collect();
    assert_eq!(process_create_order(nonce, order_uuid.clone()), Ok(()));
    assert_eq!(
        load_storage_check_nonce_in_insert_order_cache(nonce),
        Ok(false)
    );
    assert_eq!(
        lock_storage_and_check_order_in_orderbook(order_uuid),
        Ok(true)
    );
}
