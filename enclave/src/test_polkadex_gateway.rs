use log::*;
use polkadex_sgx_primitives::{AccountId, AssetId};
use polkadex_sgx_primitives::accounts::get_account;
use polkadex_sgx_primitives::types::{MarketId, Order, OrderSide, OrderType};
use sgx_tstd::vec::Vec;

// Polkadex represents 1 Token as 10^^18 minimum possible units
use crate::constants::UNIT;
use crate::polkadex::{add_main_account, create_in_memory_account_storage};
use crate::polkadex_balance_storage::{create_in_memory_balance_storage, lock_storage_and_deposit, lock_storage_and_get_balances, lock_storage_and_initialize_balance};
use crate::polkadex_gateway::{authenticate_user, place_order};
use crate::polkadex_orderbook_storage::create_in_memory_orderbook_storage;
use crate::test_proxy::initialize_dummy;

pub fn initialize_storage() {
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
        error!("Expected Reserved balance: {}, Given: {}", balance.reserved, reserved);
        return Err(1);
    }
    Ok(())
}

pub fn test_place_limit_buy_order() {
    let main: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId { base: AssetId::POLKADEX, quote: AssetId::DOT },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::DOT).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    new_order.quantity = 100 * UNIT;
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap();  // Balance: DOT = (99,1) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_err());
    new_order.quantity = UNIT;
    new_order.price = Some(99 * UNIT);
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap();  // Balance: DOT = (99,1) where (free,reserved)
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    check_balance(0u128, 100 * UNIT, main.clone(), AssetId::DOT).unwrap();  // Balance: DOT = (0,100) where (free,reserved)
}

pub fn test_place_limit_sell_order() {}

pub fn test_place_market_buy_order() {}

pub fn test_place_market_sell_order() {}

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