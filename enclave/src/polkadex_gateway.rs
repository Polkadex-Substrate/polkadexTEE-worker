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

pub extern crate alloc;
use derive_more::{Display, From};
use frame_support::ensure;
use log::*;
use polkadex_sgx_primitives::types::{
    CancelOrder, Order, OrderSide, OrderType, OrderUUID, PriceAndQuantityType, TradeEvent, UserId,
};
use polkadex_sgx_primitives::{AccountId, AssetId};
use std::sync::Arc;

use crate::accounts_nonce_storage;
use crate::constants::UNIT;
use crate::openfinex::openfinex_api::{OpenFinexApi, OpenFinexApiError};
use crate::openfinex::openfinex_types::RequestId;
use crate::polkadex_balance_storage;
use crate::polkadex_cache::cache_api::StaticStorageApi;
use crate::polkadex_cache::cancel_order_cache::CancelOrderCache;
use crate::polkadex_cache::create_order_cache::CreateOrderCache;
use crate::polkadex_gateway;
use crate::polkadex_orderbook_storage;
use crate::rpc::worker_api_direct::send_uuid;
use accounts_nonce_storage::error::Error as AccountRegistryError;

/// Trait for callbacks coming from the OpenFinex side
pub trait PolkaDexGatewayCallback {
    fn process_cancel_order(&self, order_uuid: OrderUUID) -> Result<(), GatewayError>;

    fn process_create_order(
        &self,
        request_id: RequestId,
        order_uuid: OrderUUID,
    ) -> Result<(), GatewayError>;

    fn settle_trade(&self, trade_event: TradeEvent) -> Result<(), GatewayError>;
}

/// factory to create a callback impl, allows to hide implementation (keep private)
pub struct PolkaDexGatewayCallbackFactory {}
impl PolkaDexGatewayCallbackFactory {
    pub fn create() -> Arc<dyn PolkaDexGatewayCallback> {
        Arc::new(PolkaDexGatewayCallbackImpl {})
    }
}

struct PolkaDexGatewayCallbackImpl {}

impl PolkaDexGatewayCallback for PolkaDexGatewayCallbackImpl {
    fn process_cancel_order(&self, order_uuid: OrderUUID) -> Result<(), GatewayError> {
        process_cancel_order(order_uuid)
    }

    fn process_create_order(
        &self,
        request_id: RequestId,
        order_uuid: OrderUUID,
    ) -> Result<(), GatewayError> {
        process_create_order(request_id, order_uuid)
    }

    fn settle_trade(&self, trade_event: TradeEvent) -> Result<(), GatewayError> {
        polkadex_gateway::settle_trade(trade_event)
    }
}

/// All sendings to the openfinex server should go through
/// this gateway. Necessary to mock unit tests
pub struct OpenfinexPolkaDexGateway<B: OpenFinexApi> {
    openfinex_api: B,
}
impl<B: OpenFinexApi> OpenfinexPolkaDexGateway<B> {
    pub fn new(openfinex_api: B) -> Self {
        OpenfinexPolkaDexGateway { openfinex_api }
    }
    /// Place order function does the following
    /// 1. authenticate
    /// 2. mutate balances (reserve amount offered in order)
    /// 3. store_order (async)
    /// 4. send order to OpenFinex API
    /// 5. report OpenFinex API result to sender
    pub fn place_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        order: Order,
    ) -> Result<RequestId, GatewayError> {
        debug!("Received Order: {:?}", order);
        authenticate_user(main_account.clone(), proxy_acc)?;
        basic_order_checks(&order)?;
        // Mutate Balances
        match (order.order_type, order.side) {
            (OrderType::LIMIT, OrderSide::BID) => {
                let amount = ((get_price(order.price)? as f64)
                    * ((order.quantity as f64) / (UNIT as f64)))
                    as u128;
                polkadex_balance_storage::lock_storage_and_reserve_balance(
                    &main_account,
                    order.market_id.quote,
                    amount,
                )?;
            }
            (OrderType::LIMIT, OrderSide::ASK) => {
                polkadex_balance_storage::lock_storage_and_reserve_balance(
                    &main_account,
                    order.market_id.base,
                    order.quantity,
                )?;
            }
            (OrderType::MARKET, OrderSide::BID) => {
                polkadex_balance_storage::lock_storage_and_reserve_balance(
                    &main_account,
                    order.market_id.quote,
                    get_price(order.price)?,
                )?;
            }
            (OrderType::MARKET, OrderSide::ASK) => {
                polkadex_balance_storage::lock_storage_and_reserve_balance(
                    &main_account,
                    order.market_id.base,
                    order.quantity,
                )?;
            }
            _ => {
                error!("OrderType is not implemented");
                return Err(GatewayError::NotImplementedYet);
            }
        }
        // Store the order
        // Order will be cached using incremental nonce and submitted to Openfinex with the nonce and it is stored to Orderbook
        // after nonce is replaced with OrderUUID from Openfinex
        let mutex = CreateOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
        let mut cache = match mutex.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!(
                    "Could not acquire lock on create order cache pointer: {}",
                    e
                );
                return Err(GatewayError::UnableToLock);
            }
        };

        let request_id =
            self.send_order_to_open_finex(order.clone(), cache.request_id() as RequestId)?;
        cache.insert_order(order);
        Ok(request_id)
    }

    fn send_order_to_open_finex(
        &self,
        order: Order,
        request_id: RequestId,
    ) -> Result<RequestId, GatewayError> {
        // TODO: Send order to Openfinex for inclusion ( this is a non-blocking call )
        /* let openfinex_api = OpenFinexApiImpl::new(
            OpenFinexClientInterface::new(0), // FIXME: for now hardcoded 0, but we should change that to..?
        ); */
        self.openfinex_api
            .create_order(order, request_id)
            .map_err(GatewayError::OpenFinexApiError)
    }

    fn send_cancel_request_to_openfinex(
        &self,
        cancel_order: CancelOrder,
        request_id: RequestId,
    ) -> Result<(), GatewayError> {
        /* let openfinex_api = OpenFinexApiImpl::new(
            OpenFinexClientInterface::new(0), // FIXME: for now hardcoded 0, but we should change that to..?
        ); */
        self.openfinex_api
            .cancel_order(cancel_order, request_id)
            .map_err(GatewayError::OpenFinexApiError)
    }

    /// Cancel order function does the following
    /// 1. authenticate
    /// 2. Cache the cancel request
    /// 3. send cancel_order to OpenFinex API
    pub fn cancel_order(
        &self,
        main_account: AccountId,
        proxy_acc: Option<AccountId>,
        cancel_order: CancelOrder,
    ) -> Result<(), GatewayError> {
        // Authenticate
        authenticate_user(main_account, proxy_acc)?;
        let mutex = CancelOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
        let mut cache = match mutex.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Could not acquire lock on cancel cache pointer: {}", e);
                return Err(GatewayError::UnableToLock);
            }
        };

        self.send_cancel_request_to_openfinex(
            cancel_order.clone(),
            cache.request_id() as RequestId,
        )?;
        cache.insert_order(cancel_order.order_id.clone());
        error!(">> Cache Cancel Order 1st {:?}", cancel_order);

        Ok(())
    }
}
//Only for test
// pub fn lock_storage_get_cache_nonce() -> Result<u128, GatewayError> {
//     let mutex = CreateOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
//     let cache = match mutex.lock() {
//         Ok(guard) => guard,
//         Err(e) => {
//             error!(
//                 "Could not acquire lock on create order cache pointer: {}",
//                 e
//             );
//             return Err(GatewayError::UnableToLock);
//         }
//     };
//     Ok(cache.request_id())
// }

// Only for test
pub fn _lock_storage_get_order(request_id: RequestId) -> Result<Order, GatewayError> {
    let mutex = CreateOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
    let cache = match mutex.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!(
                "Could not acquire lock on create order cache pointer: {}",
                e
            );
            return Err(GatewayError::UnableToLock);
        }
    };
    cache
        .get_order(request_id)
        .ok_or(GatewayError::NonceNotPresent)
}

// /// process_cancel_order does the following
// /// 1. Checks the orderUUID with cancel request cache
// /// 2. Remove order from Orderbook Mirror
// /// 3. Mutate the balances
pub fn process_cancel_order(order_uuid: OrderUUID) -> Result<(), GatewayError> {
    let mutex = CancelOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
    let mut cancel_cache = match mutex.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Could not acquire lock on cancel cache pointer: {}", e);
            return Err(GatewayError::UnableToLock);
        }
    };

    if !cancel_cache.remove_order(&order_uuid) {
        error!("Order Cancel Request not found in Cache");
        return Err(GatewayError::UnableToCancelOrder);
    }

    let cancelled_order = polkadex_orderbook_storage::lock_storage_and_remove_order(&order_uuid)?;
    match (cancelled_order.order_type, cancelled_order.side) {
        (OrderType::LIMIT, OrderSide::BID) => {
            let price = cancelled_order
                .price
                .ok_or(GatewayError::LimitOrderPriceNotFound)?;

            let amount =
                ((price as f64) * ((cancelled_order.quantity as f64) / (UNIT as f64))) as u128;

            polkadex_balance_storage::lock_storage_unreserve_balance(
                &cancelled_order.user_uid,
                cancelled_order.market_id.quote,
                amount,
            )?;
        }

        (OrderType::LIMIT, OrderSide::ASK) => {
            polkadex_balance_storage::lock_storage_unreserve_balance(
                &cancelled_order.user_uid,
                cancelled_order.market_id.base,
                cancelled_order.quantity,
            )?;
        }

        (OrderType::MARKET, _) => {
            error!("Cancel Order is not applicable for Market Order");
            return Err(GatewayError::UndefinedBehaviour);
        }

        (OrderType::FillOrKill | OrderType::PostOnly, _) => {
            error!("OrderType is not implemented");
            return Err(GatewayError::NotImplementedYet);
        }
    };
    Ok(())
}

pub fn process_create_order(
    request_id: RequestId,
    order_uuid: OrderUUID,
) -> Result<(), GatewayError> {
    // TODO check that this is correct @Bigna
    let mutex = CreateOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
    let mut create_cache = match mutex.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Could not acquire lock on cancel cache pointer: {}", e);
            return Err(GatewayError::UnableToLock);
        }
    };

    if let Some(order) = create_cache.remove_order(&request_id) {
        // Insert order in order book
        if let Err(e) =
            polkadex_orderbook_storage::lock_storage_and_add_order(order, order_uuid.clone())
        {
            error!("Locking storage and adding order failed. Error: {:?}", e);
            return Err(GatewayError::UnableToLock);
        };
    } else {
        return Err(GatewayError::NonceNotPresent);
    }
    if send_uuid(request_id, order_uuid).is_err() {
        return Err(GatewayError::NotAbleToSendUUID);
    }
    Ok(())
}

pub fn authenticate_user_and_validate_nonce(
    main_acc: AccountId,
    proxy_acc: Option<AccountId>,
    nonce: u32,
) -> Result<(), GatewayError> {
    accounts_nonce_storage::auth_user_validate_increment_nonce(main_acc, proxy_acc, nonce)
        .map_err(GatewayError::AccountRegistryError)
}

pub fn authenticate_user(
    main_acc: AccountId,
    proxy_acc: Option<AccountId>,
) -> Result<(), GatewayError> {
    // Authentication
    match proxy_acc {
        Some(proxy) => {
            if !accounts_nonce_storage::check_if_proxy_registered(main_acc, proxy)
                .map_err(GatewayError::AccountRegistryError)?
            {
                // FIXME: Should this really be an error?
                debug!("Proxy Account is not registered for given Main Account");
                return Err(GatewayError::ProxyNotRegisteredForMainAccount);
            }
        }
        None => {
            if !accounts_nonce_storage::check_if_main_account_registered(main_acc)
                .map_err(GatewayError::AccountRegistryError)?
            {
                // FIXME: Should this really be an error?
                debug!("Main Account is not registered");
                return Err(GatewayError::MainAccountNotRegistered);
            }
        }
    }
    Ok(())
}

//static CREATE_ORDER_NONCE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());
//static CREATE_ORDER_CACHE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());
//static CANCEL_ORDER_CACHE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn initialize_polkadex_gateway() {
    debug!("initialising polkadex gateway");
    /* let nonce: u128 = 0;
    let create_nonce_storage_ptr = Arc::new(SgxMutex::<u128>::new(nonce));
    let create_nonce_ptr = Arc::into_raw(create_nonce_storage_ptr);
    CREATE_ORDER_NONCE.store(create_nonce_ptr as *mut (), Ordering::SeqCst); */

    /* let cancel_cache: HashSet<OrderUUID> = HashSet::new();
    let cancel_cache_storage_ptr = Arc::new(SgxMutex::new(cancel_cache));
    let cancel_cache_ptr = Arc::into_raw(cancel_cache_storage_ptr);
    CANCEL_ORDER_CACHE.store(cancel_cache_ptr as *mut (), Ordering::SeqCst); */

    // TODO revisit this once these caches are using the new CacheProvider trait and implementation
    CancelOrderCache::initialize();
    CreateOrderCache::initialize();

    /* let create_cache: HashMap<u128, Order> = HashMap::new();
    let create_cache_storage_ptr = Arc::new(SgxMutex::new(create_cache));
    let create_cache_ptr = Arc::into_raw(create_cache_storage_ptr);
    CREATE_ORDER_CACHE.store(create_cache_ptr as *mut (), Ordering::SeqCst); */
}
/*
fn load_finex_nonce_pointer() -> SgxResult<&'static SgxMutex<u128>> {
    let ptr = CREATE_ORDER_NONCE.load(Ordering::SeqCst) as *mut SgxMutex<u128>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
} */

/* fn load_cancel_cache_pointer() -> SgxResult<&'static SgxMutex<HashSet<OrderUUID>>> {
    let ptr = CANCEL_ORDER_CACHE.load(Ordering::SeqCst) as *mut SgxMutex<HashSet<OrderUUID>>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
} */

/* fn load_create_cache_pointer() -> SgxResult<&'static SgxMutex<HashMap<u128, Order>>> {
    let ptr = CREATE_ORDER_CACHE.load(Ordering::SeqCst) as *mut SgxMutex<HashMap<u128, Order>>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}

fn get_finex_nonce_and_increment() -> SgxResult<u128> {
    let mutex = load_finex_nonce_pointer()?;
    let mut nonce: SgxMutexGuard<u128> = mutex.lock().unwrap();
    let current_nonce = nonce.clone();
    *nonce += 1;
    Ok(current_nonce)
}

pub fn remove_order_from_cache_and_store_in_ordermirror(
    nonce: u128,
    order_uuid: OrderUUID,
) -> SgxResult<()> {
    let mutex = load_create_cache_pointer()?;
    let cache: SgxMutexGuard<HashMap<u128, Order>> = mutex.lock().unwrap();
    if let Some(order) = cache.get(&nonce) {
        polkadex_orderbook_storage::add_order(order.clone(), order_uuid)?;
        //TODO: Remove order from cache
    } else {
        error!("Unable to find order for the given nonce");
        return Err(Default::default());
    }
    Ok(())
} */

pub fn settle_trade(trade: TradeEvent) -> Result<(), GatewayError> {
    // Check if both orders exist and get them
    let maker = polkadex_orderbook_storage::lock_storage_and_remove_order(&trade.maker_order_uuid)?;
    let taker = polkadex_orderbook_storage::lock_storage_and_remove_order(&trade.taker_order_uuid)?;
    ensure!(
        (maker.market_id == taker.market_id) & (maker.market_id == trade.market_id),
        GatewayError::MarketIdMismatch
    );
    ensure!(
        maker.side == trade.maker_side,
        GatewayError::MakerSideMismatch
    );
    // Derive buyer and seller from maker and taker

    consume_order(
        trade.clone(),
        taker,
        maker,
        trade.taker_order_uuid,
        trade.maker_order_uuid,
    )?;

    Ok(())
}

pub fn basic_order_checks(order: &Order) -> Result<(), GatewayError> {
    match (order.order_type, order.side) {
        (OrderType::LIMIT, OrderSide::BID) | (OrderType::LIMIT, OrderSide::ASK)
            if get_price(order.price)? == 0 || order.quantity == 0 =>
        {
            Err(GatewayError::QuantityOrPriceZeroInLimitOrder)
        }
        (OrderType::MARKET, OrderSide::BID) if get_price(order.price)? == 0 => {
            Err(GatewayError::PriceZeroInMarketOrder)
        }
        (OrderType::MARKET, OrderSide::ASK) if order.quantity == 0 => {
            Err(GatewayError::QuantityZeroInMarketOrder)
        }
        _ => Ok(()),
    }
}

pub fn consume_order(
    trade_event: TradeEvent,
    mut current_order: Order,
    mut counter_order: Order,
    taker_order_uuid: OrderUUID,
    maker_order_uuid: OrderUUID,
) -> Result<(), GatewayError> {
    match (current_order.order_type, current_order.side) {
        (OrderType::LIMIT, OrderSide::BID) => {
            let reserved_amount = (current_order.price.unwrap() * current_order.quantity) / UNIT;

            let current_trade_amount = (current_order.quantity * trade_event.price) / UNIT;
            let counter_trade_amount = (counter_order.quantity * trade_event.price) / UNIT;
            if let Err(e) = do_asset_exchange(&mut current_order, &mut counter_order) {
                error!("Doing asset exchange failed. Error: {:?}", e);
                return Err(GatewayError::UnableToLock);
            };

            if counter_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    counter_order,
                    maker_order_uuid,
                )?;
            }

            if current_order.quantity > 0 {
                let expected_reserved_amount =
                    (current_order.quantity * current_order.price.unwrap()) / UNIT;
                let amount_to_unreserve =
                    reserved_amount - counter_trade_amount - expected_reserved_amount;
                polkadex_balance_storage::lock_storage_unreserve_balance(
                    &current_order.user_uid,
                    current_order.market_id.quote,
                    amount_to_unreserve,
                )?;
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    current_order,
                    taker_order_uuid,
                )?;
            } else {
                let amount_to_unreserve = reserved_amount - current_trade_amount;
                polkadex_balance_storage::lock_storage_unreserve_balance(
                    &current_order.user_uid,
                    current_order.market_id.quote,
                    amount_to_unreserve,
                )?;
            }
            Ok(())
        }

        (OrderType::MARKET, OrderSide::BID) => {
            if let Err(e) = do_asset_exchange_market(&mut current_order, &mut counter_order) {
                error!("Doing asset exchange market failed. Error: {:?}", e);
                return Err(GatewayError::UnableToLock);
            };
            if counter_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    counter_order,
                    maker_order_uuid,
                )?;
            }

            if current_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    current_order,
                    taker_order_uuid,
                )?;
            }
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::ASK) => {
            let reserved_amount = (get_price(counter_order.price)? * counter_order.quantity) / UNIT;
            let counter_trade_amount = (trade_event.price * counter_order.quantity) / UNIT;
            let current_trade_amount = (current_order.quantity * trade_event.price) / UNIT;
            do_asset_exchange(&mut current_order, &mut counter_order)?;
            if counter_order.quantity > 0 {
                let required_reserved_amount =
                    (counter_order.quantity * counter_order.price.unwrap()) / UNIT;
                let amount_to_unreserve =
                    reserved_amount - current_trade_amount - required_reserved_amount;
                polkadex_balance_storage::lock_storage_unreserve_balance(
                    &counter_order.user_uid,
                    counter_order.market_id.quote,
                    amount_to_unreserve,
                )?;

                polkadex_orderbook_storage::lock_storage_and_add_order(
                    counter_order,
                    maker_order_uuid,
                )?;
            } else {
                let amount_to_unreserve = reserved_amount - counter_trade_amount;
                error!("AskLimit!");
                error!("amount_to_unreserve {:?}", amount_to_unreserve);
                polkadex_balance_storage::lock_storage_unreserve_balance(
                    &counter_order.user_uid,
                    counter_order.market_id.quote,
                    amount_to_unreserve,
                )?;
            }

            if current_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    current_order,
                    taker_order_uuid,
                )?;
            }
            Ok(())
        }

        (OrderType::MARKET, OrderSide::ASK) => {
            let reserved_amount = (get_price(counter_order.price)? * counter_order.quantity) / UNIT;
            let trade_amount = (trade_event.price * counter_order.quantity) / UNIT;
            do_asset_exchange_market(&mut current_order, &mut counter_order)?;
            if counter_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    counter_order,
                    maker_order_uuid,
                )?;
            } else {
                let amount_to_unreserve = reserved_amount - trade_amount;
                polkadex_balance_storage::lock_storage_unreserve_balance(
                    &counter_order.user_uid,
                    counter_order.market_id.quote,
                    amount_to_unreserve,
                )?;
            }

            if current_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    current_order,
                    taker_order_uuid,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn do_asset_exchange(
    current_order: &mut Order,
    counter_order: &mut Order,
) -> Result<(), GatewayError> {
    match (current_order.order_type, current_order.side) {
        (OrderType::LIMIT, OrderSide::BID) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = ((get_price(counter_order.price)? as f64)
                * ((current_order.quantity as f64) / (UNIT as f64)))
                as u128;

            transfer_asset(
                &current_order.market_id.quote,
                trade_amount,
                &current_order.user_uid,
                &counter_order.user_uid,
            )?;
            transfer_asset(
                &current_order.market_id.base,
                current_order.quantity,
                &counter_order.user_uid,
                &current_order.user_uid,
            )?;
            counter_order.quantity -= current_order.quantity;
            current_order.quantity = 0;
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::BID) if current_order.quantity > counter_order.quantity => {
            let trade_amount = ((get_price(counter_order.price)? as f64)
                * ((counter_order.quantity as f64) / (UNIT as f64)))
                as u128;

            transfer_asset(
                &current_order.market_id.quote,
                trade_amount,
                &current_order.user_uid,
                &counter_order.user_uid,
            )?;
            transfer_asset(
                &current_order.market_id.base,
                counter_order.quantity,
                &counter_order.user_uid,
                &current_order.user_uid,
            )?;
            current_order.quantity -= counter_order.quantity;
            counter_order.quantity = 0;
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::ASK) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = ((get_price(counter_order.price)? as f64)
                * ((current_order.quantity as f64) / (UNIT as f64)))
                as u128;

            transfer_asset(
                &current_order.market_id.quote,
                trade_amount,
                &counter_order.user_uid,
                &current_order.user_uid,
            )?;
            transfer_asset(
                &current_order.market_id.base,
                current_order.quantity,
                &current_order.user_uid,
                &counter_order.user_uid,
            )?;
            counter_order.quantity -= current_order.quantity;
            current_order.quantity = 0;
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::ASK) if current_order.quantity > counter_order.quantity => {
            let trade_amount = ((get_price(counter_order.price)? as f64)
                * ((counter_order.quantity as f64) / (UNIT as f64)))
                as u128;

            transfer_asset(
                &current_order.market_id.quote,
                trade_amount,
                &counter_order.user_uid,
                &current_order.user_uid,
            )?;
            transfer_asset(
                &current_order.market_id.base,
                counter_order.quantity,
                &current_order.user_uid,
                &counter_order.user_uid,
            )?;
            current_order.quantity -= counter_order.quantity;
            counter_order.quantity = 0;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn do_asset_exchange_market(
    current_order: &mut Order,
    counter_order: &mut Order,
) -> Result<(), GatewayError> {
    match (current_order.order_type, current_order.side) {
        (OrderType::MARKET, OrderSide::BID) => {
            let current_order_quantity =
                (get_price(current_order.price)? / get_price(counter_order.price)?) * UNIT;
            if current_order_quantity <= counter_order.quantity {
                transfer_asset(
                    &current_order.market_id.quote,
                    get_price(current_order.price)?,
                    &current_order.user_uid,
                    &counter_order.user_uid,
                )?;
                transfer_asset(
                    &current_order.market_id.base,
                    current_order_quantity,
                    &counter_order.user_uid,
                    &current_order.user_uid,
                )?;
                counter_order.quantity -= current_order_quantity;
                current_order.price = Some(0);
            } else {
                let trade_amount =
                    (get_price(counter_order.price)? * counter_order.quantity) / UNIT;
                transfer_asset(
                    &current_order.market_id.quote,
                    trade_amount,
                    &current_order.user_uid,
                    &counter_order.user_uid,
                )?;
                transfer_asset(
                    &current_order.market_id.base,
                    counter_order.quantity,
                    &counter_order.user_uid,
                    &current_order.user_uid,
                )?;
                counter_order.quantity = 0;
                current_order.price = Some(get_price(current_order.price)? - trade_amount);
            }
            Ok(())
        }
        (OrderType::MARKET, OrderSide::ASK) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = (get_price(counter_order.price)? * current_order.quantity) / UNIT;
            transfer_asset(
                &current_order.market_id.quote,
                trade_amount,
                &counter_order.user_uid,
                &current_order.user_uid,
            )?;
            transfer_asset(
                &current_order.market_id.base,
                current_order.quantity,
                &current_order.user_uid,
                &counter_order.user_uid,
            )?;
            counter_order.quantity -= current_order.quantity;
            current_order.quantity = 0;
            Ok(())
        }
        (OrderType::MARKET, OrderSide::ASK) if current_order.quantity > counter_order.quantity => {
            let trade_amount = (get_price(counter_order.price)? * counter_order.quantity) / UNIT;
            transfer_asset(
                &current_order.market_id.quote,
                trade_amount,
                &counter_order.user_uid,
                &current_order.user_uid,
            )?;
            transfer_asset(
                &current_order.market_id.base,
                counter_order.quantity,
                &current_order.user_uid,
                &counter_order.user_uid,
            )?;
            current_order.quantity -= counter_order.quantity;
            counter_order.quantity = 0;
            Ok(())
        }
        _ => Ok(()),
    }
}
//
pub fn transfer_asset(
    asset_id: &AssetId,
    amount: u128,
    from: &UserId,
    to: &UserId,
) -> Result<(), GatewayError> {
    polkadex_balance_storage::lock_storage_unreserve_balance(from, *asset_id, amount)?;
    polkadex_balance_storage::lock_storage_transfer_balance(from, to, *asset_id, amount)?;
    Ok(())
}

pub fn get_price(
    price: Option<PriceAndQuantityType>,
) -> Result<PriceAndQuantityType, GatewayError> {
    price.ok_or(GatewayError::PriceIsNull)
}

#[derive(Debug, Display, From, PartialEq, Eq)]
pub enum GatewayError {
    /// Price is Not Provided
    PriceIsNull,
    ///Quantity zero in MarketOrder
    QuantityZeroInMarketOrder,
    ///Price zero in MarketOrder
    PriceZeroInMarketOrder,
    ///Quantity or Price zero in LimitOrder
    QuantityOrPriceZeroInLimitOrder,
    /// Nonce not present
    NonceNotPresent,
    /// Nonce Invalid
    NonceInvalid,
    /// Price for limit Order not found
    LimitOrderPriceNotFound, // FIXME Duplicate
    /// Not implemented yet
    NotImplementedYet,
    /// Order Not found for given OrderUUID
    OrderNotFound,
    /// Proxy account not associated with Main acc
    ProxyNotRegisteredForMainAccount,
    /// Main account is not registered,
    MainAccountNotRegistered,
    /// Undefined Behaviour
    UndefinedBehaviour,
    /// Error in cancelling the order
    UnableToCancelOrder,
    /// MarketIds don't match for given trade, maker, and taker
    MarketIdMismatch,
    /// Maker OrderSide mismatch between TradeEvent and MakerOrder
    MakerSideMismatch,
    /// Unable to Load pointer
    UnableToLoadPointer,
    /// Not enough Free Balance
    NotEnoughFreeBalance,
    /// Not enough Reserved Balance,
    NotEnoughReservedBalance,
    /// Unable to find AcccountId or AssetId,
    AccountIdOrAssetIdNotFound,
    /// Could not load pointer
    NullPointer,
    /// Could acquire mutx
    UnableToLock,
    /// Error within OpenFinex api part
    OpenFinexApiError(OpenFinexApiError),
    /// Error within polkadex account registry
    AccountRegistryError(AccountRegistryError),
    /// Send UUID
    NotAbleToSendUUID,
}
