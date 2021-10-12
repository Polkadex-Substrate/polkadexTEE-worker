// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use log::*;
use polkadex_sgx_primitives::types::{
    CancelOrder, MarketId, Order, OrderSide, OrderType, OrderUUID, TradeEvent,
};
use polkadex_sgx_primitives::{accounts::get_account, AccountId, AssetId};
use sgx_tstd::vec::Vec;

// Polkadex represents 1 Token as 10^^18 minimum possible units
use crate::accounts_nonce_storage::test_proxy::initialize_dummy;
use crate::accounts_nonce_storage::{
    add_main_account, check_if_main_account_registered, create_in_memory_accounts_and_nonce_storage,
};
use crate::openfinex::openfinex_api::{OpenFinexApi, OpenFinexApiResult};
use crate::polkadex_balance_storage::{
    create_in_memory_balance_storage, lock_storage_and_deposit, lock_storage_and_get_balances,
    lock_storage_and_initialize_balance,
};
use crate::polkadex_cache::cache_api::RequestId;
use crate::polkadex_gateway::{
    authenticate_user, initialize_polkadex_gateway, settle_trade, GatewayError,
    OpenfinexPolkaDexGateway,
};
use crate::polkadex_orderbook_storage::{
    create_in_memory_orderbook_storage, lock_storage_and_add_order,
};
use substratee_settings::node::UNIT;

pub struct OpenFinexApiMock {}

impl OpenFinexApiMock {
    pub fn new() -> Self {
        OpenFinexApiMock {}
    }
}

impl OpenFinexApi for OpenFinexApiMock {
    fn create_order(&self, _order: Order, _request_id: RequestId) -> OpenFinexApiResult<RequestId> {
        Ok(0u128)
    }

    fn cancel_order(
        &self,
        _cancel_order: CancelOrder,
        _request_id: RequestId,
    ) -> OpenFinexApiResult<()> {
        Ok(())
    }

    fn withdraw_funds(&self, _request_id: RequestId) -> OpenFinexApiResult<()> {
        Ok(())
    }

    fn deposit_funds(&self, _request_id: RequestId) -> OpenFinexApiResult<()> {
        Ok(())
    }
}

pub fn create_mock_gateway() -> OpenfinexPolkaDexGateway<OpenFinexApiMock> {
    OpenfinexPolkaDexGateway::new(OpenFinexApiMock::new())
}

pub fn initialize_storage() {
    // Initialize Gateway
    initialize_polkadex_gateway();
    // Initialize Account Storage
    create_in_memory_accounts_and_nonce_storage(vec![]);
    // Initialize Balance storage
    assert!(create_in_memory_balance_storage().is_ok());
    // Initialize Order Mirror
    assert!(create_in_memory_orderbook_storage(vec![]).is_ok());
    initialize_polkadex_gateway();
}

fn setup(main: AccountId) {
    // Check if account is already registered
    if let Ok(false) = check_if_main_account_registered(main.clone()) {
        // Register Main account
        assert!(add_main_account(main.clone()).is_ok());
    }
    // Initialize Balance for main account
    assert!(lock_storage_and_initialize_balance(main.clone(), AssetId::POLKADEX).is_ok());
    assert!(lock_storage_and_initialize_balance(main.clone(), AssetId::Asset(0)).is_ok());
    // Deposit some balance
    assert!(lock_storage_and_deposit(main.clone(), AssetId::POLKADEX, 100 * UNIT).is_ok());
    assert!(lock_storage_and_deposit(main, AssetId::Asset(0), 100 * UNIT).is_ok());
}

pub fn check_balance(
    free: u128,
    reserved: u128,
    main: AccountId,
    token: AssetId,
) -> Result<(), u32> {
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
//
pub fn test_place_limit_buy_order() {
    let gateway = create_mock_gateway();
    let main: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::Asset(0),
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::Asset(0)).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::Asset(0)).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::Asset(0)).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    new_order.price = Some(99 * UNIT);
    assert!(gateway.place_order(main.clone(), None, new_order).is_ok());
    check_balance(0u128, 100 * UNIT, main.clone(), AssetId::Asset(0)).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main, AssetId::POLKADEX).unwrap();
}

pub fn test_place_limit_sell_order() {
    let gateway = create_mock_gateway();
    let main: AccountId = get_account("test_place_limit_sell_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::Asset(0),
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::ASK,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    new_order.price = Some(99 * UNIT);
    assert!(gateway.place_order(main.clone(), None, new_order).is_ok());
    check_balance(98 * UNIT, 2 * UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main, AssetId::Asset(0)).unwrap();
}

pub fn test_place_market_buy_order() {
    let gateway = create_mock_gateway();
    let main: AccountId = get_account("test_place_market_buy_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::Asset(0),
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::Asset(0)).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::Asset(0)).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.price = Some(100 * UNIT);
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::Asset(0)).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.price = Some(99 * UNIT);
    assert!(gateway.place_order(main.clone(), None, new_order).is_ok());
    check_balance(0u128, 100 * UNIT, main.clone(), AssetId::Asset(0)).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main, AssetId::POLKADEX).unwrap();
}

pub fn test_place_market_sell_order() {
    let gateway = create_mock_gateway();
    let main: AccountId = get_account("test_place_market_sell_order");
    let mut new_order: Order = Order {
        user_uid: main.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::Asset(0),
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::ASK,
        quantity: UNIT,
        price: Some(UNIT),
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    assert!(gateway.place_order(main.clone(), None, new_order).is_ok());
    check_balance(98 * UNIT, 2 * UNIT, main.clone(), AssetId::POLKADEX).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main, AssetId::Asset(0)).unwrap();
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

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_ask_limit() {
    let gateway = create_mock_gateway();
    // BUY LIMIT ORDER
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let mut new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::Asset(0),
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::Asset(0)),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::Asset(0)),
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
            quote: AssetId::Asset(0),
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
    assert!(gateway
        .place_order(sell_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user, AssetId::POLKADEX),
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
            quote: AssetId::Asset(0),
        },
        trade_id: 1,
        price: 2 * UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: buy_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: buy_order_uuid,
        taker_user_id: sell_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: sell_order_uuid,
        maker_side: OrderSide::BID,
        timestamp: 23,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user, AssetId::Asset(0)),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 0u128, buy_order_user, AssetId::Asset(0)),
        Ok(())
    );
}

fn setup_btc_usd(account: AccountId) {
    // Check if account is already registered
    if let Ok(false) = check_if_main_account_registered(account.clone()) {
        // Register Main account
        assert!(add_main_account(account.clone()).is_ok());
    }
    // Initialize Balance for main account
    assert!(
        lock_storage_and_initialize_balance(account.clone(), AssetId::Asset(4294967297)).is_ok()
    );
    assert!(lock_storage_and_initialize_balance(account.clone(), AssetId::Asset(840)).is_ok());
    // Deposit some balance
    assert!(lock_storage_and_deposit(account.clone(), AssetId::Asset(840), 100 * UNIT).is_ok());
    assert!(lock_storage_and_deposit(account, AssetId::Asset(4294967297), 10 * UNIT).is_ok());
}

fn place_order(
    trader: AccountId,
    order_type: OrderType,
    order_side: OrderSide,
    price: u128,
    quanity: u128,
    order_id: OrderUUID,
) -> Result<(), GatewayError> {
    let gateway = create_mock_gateway();
    let new_order: Order = Order {
        user_uid: trader.clone(),
        market_id: MarketId {
            base: AssetId::Asset(4294967297),
            quote: AssetId::Asset(840),
        },
        market_type: Vec::from("trusted"),
        order_type,
        side: order_side,
        quantity: quanity,
        price: Some(price),
    };

    gateway.place_order(trader, None, new_order.clone())?;
    lock_storage_and_add_order(new_order, order_id)?;
    Ok(())
}

#[allow(unused)]
pub fn test_orderbook_limit() {
    let gateway = create_mock_gateway();
    let alice: AccountId = get_account("Alice");
    let bob: AccountId = get_account("Bob");
    setup_btc_usd(alice.clone());
    setup_btc_usd(bob.clone());
    let alice_sell_order_uuid: OrderUUID = (0..100).collect();
    place_order(
        alice.clone(),
        OrderType::LIMIT,
        OrderSide::ASK,
        5 * UNIT,
        3 * UNIT,
        alice_sell_order_uuid.clone(),
    );
    assert_eq!(
        check_balance(100 * UNIT, 0, alice.clone(), AssetId::Asset(840)),
        Ok(())
    );
    assert_eq!(
        check_balance(
            7 * UNIT,
            3 * UNIT,
            alice.clone(),
            AssetId::Asset(4294967297)
        ),
        Ok(())
    );
    let bob_buy_order_uuid: OrderUUID = (10..59).collect();
    place_order(
        bob.clone(),
        OrderType::LIMIT,
        OrderSide::BID,
        8 * UNIT,
        4 * UNIT,
        bob_buy_order_uuid.clone(),
    );

    let order_event = TradeEvent {
        market_id: MarketId {
            base: AssetId::Asset(4294967297),
            quote: AssetId::Asset(840),
        },
        trade_id: 1,
        price: 5 * UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: alice.clone(),
        maker_order_id: 1,
        maker_order_uuid: alice_sell_order_uuid,
        taker_user_id: bob.clone(),
        taker_order_id: 2,
        taker_order_uuid: bob_buy_order_uuid,
        maker_side: OrderSide::ASK,
        timestamp: 23,
    };
    assert_eq!(settle_trade(order_event), Ok(()));

    assert_eq!(
        check_balance(115 * UNIT, 0, alice.clone(), AssetId::Asset(840)),
        Ok(())
    );
    assert_eq!(
        check_balance(7 * UNIT, 0, alice, AssetId::Asset(4294967297)),
        Ok(())
    );

    assert_eq!(
        check_balance(77 * UNIT, 8 * UNIT, bob.clone(), AssetId::Asset(840)),
        Ok(())
    );
    assert_eq!(
        check_balance(13 * UNIT, 0, bob, AssetId::Asset(4294967297)),
        Ok(())
    );
}
