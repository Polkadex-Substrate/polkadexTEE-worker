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
use crate::accounts_nonce_storage::{
    add_main_account, check_if_main_account_registered,
    create_in_memory_accounts_and_nonce_storage, test_proxy::initialize_dummy,
};
use crate::constants::UNIT;
use crate::openfinex::openfinex_api::{OpenFinexApi, OpenFinexApiResult};
use crate::polkadex_balance_storage::{
    create_in_memory_balance_storage, lock_storage_and_deposit, lock_storage_and_get_balances,
    lock_storage_and_initialize_balance,
};
use crate::polkadex_cache::cache_api::{RequestId, StaticStorageApi};
use crate::polkadex_cache::create_order_cache::CreateOrderCache;
use crate::polkadex_gateway::{
    authenticate_user, initialize_polkadex_gateway, process_cancel_order, process_create_order,
    settle_trade, GatewayError, OpenfinexPolkaDexGateway,
};
use crate::polkadex_orderbook_storage::{
    create_in_memory_orderbook_storage, lock_storage_and_add_order,
    lock_storage_and_check_order_in_orderbook, lock_storage_and_get_order,
};

pub struct OpenFinexApiMock {}

impl OpenFinexApiMock {
    pub fn new() -> Self {
        OpenFinexApiMock {}
    }
}

impl OpenFinexApi for OpenFinexApiMock {
    fn create_order(&self, _order: Order, _request_id: RequestId) -> OpenFinexApiResult<RequestId> {
        Ok(0 as RequestId)
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
    assert!(create_in_memory_accounts_and_nonce_storage(vec![]).is_ok());
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
    assert!(lock_storage_and_initialize_balance(main.clone(), AssetId::DOT).is_ok());
    // Deposit some balance
    assert!(lock_storage_and_deposit(main.clone(), AssetId::POLKADEX, 100 * UNIT).is_ok());
    assert!(lock_storage_and_deposit(main, AssetId::DOT, 100 * UNIT).is_ok());
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

pub fn test_place_limit_buy_order() {
    let gateway = create_mock_gateway();
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
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = 100 * UNIT;
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.quantity = UNIT;
    new_order.price = Some(99 * UNIT);
    assert!(gateway.place_order(main.clone(), None, new_order).is_ok());
    check_balance(0u128, 100 * UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main, AssetId::POLKADEX).unwrap();
}

pub fn test_place_limit_sell_order() {
    let gateway = create_mock_gateway();
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
    check_balance(100 * UNIT, 0u128, main, AssetId::DOT).unwrap();
}

pub fn test_place_market_buy_order() {
    let gateway = create_mock_gateway();
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
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.price = Some(100 * UNIT);
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_err());
    check_balance(99 * UNIT, UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
    new_order.price = Some(99 * UNIT);
    assert!(gateway.place_order(main.clone(), None, new_order).is_ok());
    check_balance(0u128, 100 * UNIT, main.clone(), AssetId::DOT).unwrap(); // Balance: DOT = (0,100) where (free,reserved)
    check_balance(100 * UNIT, 0u128, main, AssetId::POLKADEX).unwrap();
}

pub fn test_place_market_sell_order() {
    let gateway = create_mock_gateway();
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
    check_balance(100 * UNIT, 0u128, main, AssetId::DOT).unwrap();
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
    let gateway = create_mock_gateway();
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
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
            quote: AssetId::DOT,
        },
        trade_id: 1,
        price: UNIT,
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
        check_balance(101 * UNIT, 0u128, sell_order_user, AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(99 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_ask_limit() {
    let gateway = create_mock_gateway();
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
        quantity: 2 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(96 * UNIT, 4 * UNIT, buy_order_user, AssetId::DOT),
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
        trade_id: 1,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: buy_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: buy_order_uuid.clone(),
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
        check_balance(101 * UNIT, 0u128, sell_order_user, AssetId::DOT),
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
        user_uid: buy_order_user,
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

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_ask_limit() {
    let gateway = create_mock_gateway();
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
        quantity: UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
    assert!(gateway
        .place_order(sell_order_user.clone(), None, new_order.clone())
        .is_ok());
    error!("temp4");
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, sell_order_user, AssetId::POLKADEX,),
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
        trade_id: 1,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: buy_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: buy_order_uuid,
        taker_user_id: sell_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: sell_order_uuid.clone(),
        maker_side: OrderSide::BID,
        timestamp: 23,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(98 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX,),
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
        check_balance(99 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user,
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

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

// ALL BUY LIMIT TEST

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_buy_limit() {
    let gateway = create_mock_gateway();
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
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
        trade_id: 1,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: sell_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: sell_order_uuid,
        taker_user_id: buy_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: buy_order_uuid,
        maker_side: OrderSide::ASK,
        timestamp: 255,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user, AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(99 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_buy_limit() {
    let gateway = create_mock_gateway();
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
        quantity: 2 * UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(96 * UNIT, 4 * UNIT, buy_order_user, AssetId::DOT),
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
        trade_id: 1,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: sell_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: sell_order_uuid,
        taker_user_id: buy_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: buy_order_uuid.clone(),
        maker_side: OrderSide::ASK,
        timestamp: 623,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user, AssetId::DOT),
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
        user_uid: buy_order_user,
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

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_buy_limit() {
    let gateway = create_mock_gateway();
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
        quantity: UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    error!("tesm1");
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    error!("temp2");
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
    assert!(gateway
        .place_order(sell_order_user.clone(), None, new_order.clone())
        .is_ok());
    error!("temp4");
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, sell_order_user, AssetId::POLKADEX,),
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
        trade_id: 1,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: sell_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: sell_order_uuid.clone(),
        taker_user_id: buy_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: buy_order_uuid,
        maker_side: OrderSide::ASK,
        timestamp: 3465,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(98 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX,),
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
        check_balance(99 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user,
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

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}
//ALL SELL MARKET ORDER

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_ask_market() {
    let gateway = create_mock_gateway();
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

        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
        side: OrderSide::ASK,
        quantity: UNIT,
        price: None,
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main, AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
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
        trade_id: 7,
        price: 2 * UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: buy_order_user.clone(),
        maker_order_id: 4,
        maker_order_uuid: buy_order_uuid,
        taker_user_id: sell_order_user.clone(),
        taker_order_id: 5,
        taker_order_uuid: sell_order_uuid,
        maker_side: OrderSide::BID,
        timestamp: 134,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user, AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_ask_market() {
    let gateway = create_mock_gateway();
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
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(96 * UNIT, 4 * UNIT, buy_order_user, AssetId::DOT),
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
        side: OrderSide::ASK,
        quantity: UNIT,
        price: None,
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(99 * UNIT, UNIT, main, AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
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
        trade_id: 1,
        price: 2 * UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: buy_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: buy_order_uuid.clone(),
        taker_user_id: sell_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: sell_order_uuid,
        maker_side: OrderSide::BID,
        timestamp: 2345,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user, AssetId::DOT),
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
        user_uid: buy_order_user,
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

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_ask_market() {
    let gateway = create_mock_gateway();
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
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
        side: OrderSide::ASK,
        quantity: 2 * UNIT,
        price: None,
    };

    setup(main.clone());
    check_balance(100 * UNIT, 0u128, main.clone(), AssetId::POLKADEX).unwrap(); // Balance:  DOT = (100,0) where (free,reserved)
    assert!(gateway
        .place_order(main.clone(), None, new_order.clone())
        .is_ok());
    check_balance(98 * UNIT, 2 * UNIT, main, AssetId::POLKADEX).unwrap(); // Balance: DOT = (99,1) where (free,reserved)
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
        trade_id: 12,
        price: 2 * UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: buy_order_user.clone(),
        maker_order_id: 2354,
        maker_order_uuid: buy_order_uuid,
        taker_user_id: sell_order_user.clone(),
        taker_order_id: 324652,
        taker_order_uuid: sell_order_uuid.clone(),
        maker_side: OrderSide::BID,
        timestamp: 724524,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(98 * UNIT, UNIT, sell_order_user.clone(), AssetId::POLKADEX,),
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
        check_balance(98 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user,
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::ASK,
        quantity: UNIT,
        price: None,
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

//ALL BUY MARKET ORDER

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_full_buy_market() {
    let gateway = create_mock_gateway();
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
        quantity: UNIT,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
    assert!(gateway
        .place_order(sell_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, sell_order_user, AssetId::POLKADEX,),
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
        trade_id: 2345,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: sell_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: sell_order_uuid,
        taker_user_id: buy_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: buy_order_uuid,
        maker_side: OrderSide::ASK,
        timestamp: 6315435,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(98 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, sell_order_user, AssetId::DOT),
        Ok(())
    );
    assert_eq!(
        check_balance(102 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_bid_market() {
    let gateway = create_mock_gateway();
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
        quantity: 2 * UNIT, //Remove later
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
        trade_id: 23,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: sell_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: sell_order_uuid,
        taker_user_id: buy_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: buy_order_uuid.clone(),
        maker_side: OrderSide::ASK,
        timestamp: 265246,
    };
    assert_eq!(settle_trade(order_event), Ok(()));
    assert_eq!(
        check_balance(99 * UNIT, 0u128, sell_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(101 * UNIT, 0u128, sell_order_user, AssetId::DOT),
        Ok(())
    ); //101
    assert_eq!(
        check_balance(101 * UNIT, 0u128, buy_order_user.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        check_balance(98 * UNIT, UNIT, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(buy_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: buy_order_user,
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        quantity: 2 * UNIT,
        price: Some(UNIT),
    };

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

#[allow(unused)]
pub fn setup_place_buy_and_sell_order_partial_two_bid_market() {
    let gateway = create_mock_gateway();
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
        quantity: 2 * UNIT, //Remove later
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
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
    assert!(gateway
        .place_order(sell_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(97 * UNIT, 3 * UNIT, sell_order_user, AssetId::POLKADEX,),
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
        trade_id: 3,
        price: UNIT,
        amount: 0,
        funds: 0,
        maker_user_id: sell_order_user.clone(),
        maker_order_id: 1,
        maker_order_uuid: sell_order_uuid.clone(),
        taker_user_id: buy_order_user.clone(),
        taker_order_id: 2,
        taker_order_uuid: buy_order_uuid,
        maker_side: OrderSide::ASK,
        timestamp: 2345,
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
        check_balance(98 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );

    //CHECK FOR LEFT BUY ORDER
    let actual_order = lock_storage_and_get_order(sell_order_uuid).unwrap();

    let expected_order = Order {
        user_uid: sell_order_user,
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

    assert_eq!(actual_order, expected_order);

    //TODO Also check if ask order is removed or not
}

pub fn setup_test_cancel_limit_bid_order() {
    let gateway = create_mock_gateway();
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let new_order: Order = Order {
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
    assert!(gateway
        .place_order(buy_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(98 * UNIT, 2 * UNIT, buy_order_user, AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    let buy_order_uuid: OrderUUID = (0..100).collect();
    assert!(lock_storage_and_add_order(new_order, buy_order_uuid).is_ok());
}

pub fn test_cancel_limit_bid_order() {
    let gateway = create_mock_gateway();
    setup_test_cancel_limit_bid_order();
    let buy_order_uuid: OrderUUID = (0..100).collect();
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order");
    let order = CancelOrder {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        order_id: buy_order_uuid.clone(),
    };
    assert_eq!(
        gateway.cancel_order(buy_order_user.clone(), None, order),
        Ok(())
    );
    assert!(process_cancel_order(buy_order_uuid).is_ok());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user, AssetId::DOT),
        Ok(())
    );
}

pub fn setup_test_cancel_ask_order() {
    let gateway = create_mock_gateway();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let new_order: Order = Order {
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
    assert!(gateway
        .place_order(sell_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user, AssetId::POLKADEX),
        Ok(())
    );
    let sell_order_uuid: OrderUUID = (200..201).collect();
    assert!(lock_storage_and_add_order(new_order, sell_order_uuid).is_ok());
}

pub fn test_cancel_ask_order() {
    let gateway = create_mock_gateway();
    setup_test_cancel_ask_order();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let sell_order_uuid: OrderUUID = (200..201).collect();
    let order = CancelOrder {
        user_uid: sell_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        order_id: sell_order_uuid.clone(),
    };
    assert_eq!(
        gateway.cancel_order(sell_order_user.clone(), None, order),
        Ok(())
    );
    assert!(process_cancel_order(sell_order_uuid).is_ok());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, sell_order_user, AssetId::POLKADEX,),
        Ok(())
    );
}

pub fn setup_process_create_order() {
    let gateway = create_mock_gateway();
    let sell_order_user: AccountId = get_account("test_place_limit_sell_order_partial");
    let new_order: Order = Order {
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
    assert!(gateway
        .place_order(sell_order_user.clone(), None, new_order.clone())
        .is_ok());
    assert_eq!(
        check_balance(99 * UNIT, UNIT, sell_order_user, AssetId::POLKADEX),
        Ok(())
    );
    let request_id = insert_order_into_cache(new_order).unwrap();
    assert_eq!(
        load_storage_check_id_in_insert_order_cache(request_id),
        Ok(true)
    );
}

pub fn test_process_create_order() {
    setup_process_create_order();
    let request_id: u128 = 1;
    let order_uuid: OrderUUID = (200..201).collect();
    // In test mode, process_create_order can't send order to Worker. Hence, it is expecyed to get this error
    assert_eq!(
        process_create_order(request_id, order_uuid.clone()),
        Err(GatewayError::NotAbleToSendUUID)
    );
    assert_eq!(
        load_storage_check_id_in_insert_order_cache(request_id),
        Ok(false)
    );
    assert_eq!(
        lock_storage_and_check_order_in_orderbook(order_uuid),
        Ok(true)
    );
}

fn insert_order_into_cache(order: Order) -> Result<RequestId, GatewayError> {
    let mutex = CreateOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
    let mut create_cache = match mutex.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Could not acquire lock on cancel cache pointer: {}", e);
            return Err(GatewayError::UnableToLock);
        }
    };
    let current_request_id = create_cache.request_id();
    create_cache.insert_order(order);
    Ok(current_request_id)
}

fn load_storage_check_id_in_insert_order_cache(
    request_id: RequestId,
) -> Result<bool, GatewayError> {
    let mutex = CreateOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
    let mut create_cache = match mutex.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Could not acquire lock on cancel cache pointer: {}", e);
            return Err(GatewayError::UnableToLock);
        }
    };
    if let Some(_order) = create_cache.remove_order(&request_id) {
        return Ok(true);
    }
    Ok(false)
}

pub fn test_basic_order_checks() {
    let gateway = create_mock_gateway();
    let buy_order_user: AccountId = get_account("test_place_limit_buy_order_partial");
    let new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::LIMIT,
        side: OrderSide::BID,
        quantity: 0,
        price: Some(2 * UNIT),
    };

    setup(buy_order_user.clone());
    assert_eq!(
        check_balance(100 * UNIT, 0u128, buy_order_user.clone(), AssetId::DOT),
        Ok(())
    ); // Balance:  DOT = (100,0) where (free,reserved, Ok(())))
    assert_eq!(
        gateway.place_order(buy_order_user.clone(), None, new_order),
        Err(GatewayError::QuantityOrPriceZeroInLimitOrder)
    );

    // BID MARKET ORDER

    let new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        quantity: 0,
        price: Some(0),
    };

    assert_eq!(
        gateway.place_order(buy_order_user.clone(), None, new_order),
        Err(GatewayError::PriceZeroInMarketOrder)
    );

    let new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::ASK,
        quantity: 0,
        price: Some(0),
    };

    assert_eq!(
        gateway.place_order(buy_order_user.clone(), None, new_order),
        Err(GatewayError::QuantityZeroInMarketOrder)
    );

    let new_order: Order = Order {
        user_uid: buy_order_user.clone(),
        market_id: MarketId {
            base: AssetId::POLKADEX,
            quote: AssetId::DOT,
        },
        market_type: Vec::from("trusted"),
        order_type: OrderType::MARKET,
        side: OrderSide::BID,
        quantity: 0,
        price: Some(0),
    };

    assert_eq!(
        gateway.place_order(buy_order_user, None, new_order),
        Err(GatewayError::PriceZeroInMarketOrder)
    );
}
