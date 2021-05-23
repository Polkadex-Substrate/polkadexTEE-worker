use log::*;
use polkadex_sgx_primitives::{AccountId, AssetId};
use polkadex_sgx_primitives::accounts::get_account;
use polkadex_sgx_primitives::types::{MarketId, Order, OrderSide, OrderType};
use sgx_tstd::vec::Vec;

use crate::polkadex::{add_main_account, create_in_memory_account_storage};
use crate::polkadex_balance_storage::{create_in_memory_balance_storage, lock_storage_and_deposit, lock_storage_and_initialize_balance};
use crate::polkadex_gateway::{authenticate_user, place_order};
use crate::polkadex_orderbook_storage::create_in_memory_orderbook_storage;
use crate::test_proxy::initialize_dummy;

// Polkadex represents 1 Token as 10^^18 minimum possible units
const UNIT: u128 = 100000000000000000;

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

    // Balance:  DOT = (100,0) where (free,reserved)
    if let Err(e) = place_order(main.clone(), None, new_order.clone()) {
        error!("Panicked due to error: {:?}", e)
    }
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(place_order(main.clone(), None, new_order.clone()).is_err());
    // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    new_order.price = Some(99 * UNIT);
    assert!(place_order(main.clone(), None, new_order.clone()).is_ok());
    // Balance: DOT = (0,100) where (free,reserved)
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