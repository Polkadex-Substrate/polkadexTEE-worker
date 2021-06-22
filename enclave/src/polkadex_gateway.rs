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
use alloc::fmt::Result as FormatResult;
use frame_support::ensure;
use log::*;
use polkadex_sgx_primitives::types::{
    CancelOrder, Order, OrderSide, OrderType, OrderUUID, TradeEvent, UserId,
};
use polkadex_sgx_primitives::{AccountId, AssetId, Balance};
use std::sync::Arc;

use crate::constants::UNIT;
use crate::openfinex::openfinex_api::{OpenFinexApi, OpenFinexApiError};
use crate::openfinex::openfinex_types::RequestId;
use crate::polkadex;
use crate::polkadex_balance_storage;
use crate::polkadex_cache::cache_api::StaticStorageApi;
use crate::polkadex_cache::cancel_order_cache::CancelOrderCache;
use crate::polkadex_cache::create_order_cache::CreateOrderCache;
use crate::polkadex_orderbook_storage;
use polkadex::AccountRegistryError;

#[derive(Eq, Debug, PartialOrd, PartialEq)]
pub enum GatewayError {
	///Quantity zero in MarketOrder
	QuantityZeroInMarketOrder,
	///Price zero in MarketOrder
	PriceZeroInMarketOrder,
	///Quantity or Price zero in LimitOrder
	QuantityOrPriceZeroInLimitOrder,
    /// Nonce not present
    NonceNotPresent,
    /// TradeAmountIsNotAsExpected
    TradeAmountIsNotAsExpected,
    /// Trade amount is not as expected
    BasicOrderCheckError,
    /// Price for limit Order not found
    LimitOrderPriceNotFound,
    /// Quantity zero for limit order,
    QuantityZeroInLimitOrder,
    /// Not implemented yet
    NotImplementedYet,
    /// Order Not found for given OrderUUID
    OrderNotFound,
    /// Proxy account not associated with Main acc
    ProxyNotRegisteredForMainAccount,
    /// Main account is not registered,
    MainAccountNotRegistered,
    /// Failed to reserve balance,
    FailedToReserveBalance,
    /// Failed to Unreserve balance,
    FailedToUnReserveBalance,
    /// Unable to remove order from orderbook storage
    UnableToRemoveOrder,
    /// Undefined Behaviour
    UndefinedBehaviour,
    /// Price not defined for a market buy order
    MarketOrderPriceNotDefined,
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
}

impl alloc::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> FormatResult {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

/// Trait for callbacks coming from the OpenFinex side
pub trait PolkaDexGatewayCallback {
    fn process_cancel_order(&self, order_uuid: OrderUUID) -> Result<(), GatewayError>;

    fn process_create_order(
        &self,
        request_id: RequestId,
        order_uuid: OrderUUID,
    ) -> Result<(), GatewayError>;
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
    ) -> Result<(), GatewayError> {
        // Authentication
        if let Err(e) = authenticate_user(main_account.clone(), proxy_acc) {
            error!("Could not authenticate user due to: {:?}", e);
            return Err(e);
        };
		basic_order_checks(&order)?;
        // Mutate Balances
        match order.order_type {
            OrderType::LIMIT => {
                if order.quantity == 0 as Balance {
                    error!("Limit Order quantity Zero");
                    return Err(GatewayError::QuantityZeroInLimitOrder);
                }
                if let Some(price) = order.price {
                    match order.side {
                        OrderSide::BID => {
                            let amount = ((price as f64)
                                * ((order.quantity as f64) / (UNIT as f64)))
                                as u128;
                            match polkadex_balance_storage::lock_storage_and_reserve_balance(
                                &main_account,
                                order.market_id.quote,
                                amount,
                            ) {
                                Ok(()) => {}
                                Err(_) => return Err(GatewayError::FailedToReserveBalance),
                            };
                        }
                        OrderSide::ASK => {
                            match polkadex_balance_storage::lock_storage_and_reserve_balance(
                                &main_account,
                                order.market_id.base,
                                order.quantity,
                            ) {
                                Ok(()) => {}
                                Err(_) => return Err(GatewayError::FailedToReserveBalance),
                            };
                        }
                    }
                } else {
                    error!("Price not given for a limit order");
                    return Err(GatewayError::LimitOrderPriceNotFound);
                }
            }
            OrderType::MARKET => {
                match order.side {
                    // User defines the max amount in quote they want to use for market buy, it is defined in price field of Order.
                    OrderSide::BID => {
                        if let Some(price) = order.price {
                            match polkadex_balance_storage::lock_storage_and_reserve_balance(
                                &main_account,
                                order.market_id.quote,
                                price,
                            ) {
                                Ok(()) => {}
                                Err(_) => return Err(GatewayError::FailedToReserveBalance),
                            };
                        } else {
                            return Err(GatewayError::MarketOrderPriceNotDefined);
                        }
                    }
                    OrderSide::ASK => {
                        match polkadex_balance_storage::lock_storage_and_reserve_balance(
                            &main_account,
                            order.market_id.base,
                            order.quantity,
                        ) {
                            Ok(()) => {}
                            Err(_) => return Err(GatewayError::FailedToReserveBalance),
                        };
                    }
                }
            }
            OrderType::FillOrKill | OrderType::PostOnly => {
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

        self.send_order_to_open_finex(order.clone(), cache.request_id() as RequestId)?;
        cache.insert_order(order);
        Ok(())
    }

    fn send_order_to_open_finex(
        &self,
        order: Order,
        request_id: RequestId,
    ) -> Result<(), GatewayError> {
        // TODO: Send order to Openfinex for inclusion ( this is a non-blocking call )
        /* let openfinex_api = OpenFinexApiImpl::new(
            OpenFinexClientInterface::new(0), // FIXME: for now hardcoded 0, but we should change that to..?
        ); */
        self.openfinex_api
            .create_order(order, request_id)
            .map_err(|e| GatewayError::OpenFinexApiError(e))
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
            .map_err(|e| GatewayError::OpenFinexApiError(e))
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
        authenticate_user(main_account.clone(), proxy_acc)?;
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
        cache.insert_order(cancel_order.order_id);

        Ok(())
    }
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

    // TODO @gautham please verify cancel order logic
    // Mutate Balances
    //     if let Ok(result) = polkadex_orderbook_storage::remove_order(&order_uuid) {
    //         match result {
    //             Some(cancelled_order) => match cancelled_order.order_type {
    //                 OrderType::LIMIT => {
    //                     if let Some(price) = cancelled_order.price {
    //                         match cancelled_order.side {
    //                             OrderSide::BID => {
    //                                 let amount = ((price as f64)
    //                                     * ((cancelled_order.quantity as f64) / (UNIT as f64)))
    //                                     as u128;
    //                                 match polkadex_balance_storage::lock_storage_unreserve_balance(
    //                                     cancelled_order.user_uid,
    //                                     cancelled_order.market_id.quote,
    //                                     amount,
    //                                 ) {
    //                                     Ok(()) => {}
    //                                     Err(_) => return Err(GatewayError::FailedToUnReserveBalance),
    //                                 };
    //                             }
    //                             OrderSide::ASK => {
    //                                 match polkadex_balance_storage::lock_storage_unreserve_balance(
    //                                     cancelled_order.user_uid,
    //                                     cancelled_order.market_id.base,
    //                                     cancelled_order.quantity,
    //                                 ) {
    //                                     Ok(()) => {}
    //                                     Err(_) => return Err(GatewayError::FailedToUnReserveBalance),
    //                                 };
    //                             }
    //                         }
    //                     } else {
    //                         error!("Unable to find price for limit order");
    //                         return Err(GatewayError::LimitOrderPriceNotFound);
    //                     }
    //                 }
    //                 OrderType::MARKET => {
    //                     error!("Cancel Order is not applicable for Market Order");
    //                     return Err(GatewayError::UndefinedBehaviour);
    //                 }
    //                 OrderType::FillOrKill | OrderType::PostOnly => {
    //                     error!("OrderType is not implemented");
    //                     return Err(GatewayError::NotImplementedYet);
    //                 }
    //             },
    //             None => {
    //                 error!("Unable to find order for given order_uuid");
    //                 return Err(GatewayError::OrderNotFound);
    //             }
    //         }
    //     } else {
    //         return Err(GatewayError::UnableToRemoveOrder);
    //     }
    //     error!("Unable to load the cancel cache pointer");
    //     return Err(GatewayError::UndefinedBehaviour);
    // }
    Ok(())
}

pub fn process_create_order(nonce: u128, order_uuid: OrderUUID) -> Result<(), GatewayError> {
    // TODO check that this is correct @Bigna
    let mutex = CreateOrderCache::load().map_err(|_| GatewayError::NullPointer)?;
    let mut create_cache = match mutex.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Could not acquire lock on cancel cache pointer: {}", e);
            return Err(GatewayError::UnableToLock);
        }
    };

    if let Some(order) = create_cache.remove_order(&nonce) {
        // Insert order in order book
        if let Err(e) = polkadex_orderbook_storage::lock_storage_and_add_order(order, order_uuid) {
            error!("Locking storage and adding order failed. Error: {:?}", e);
            return Err(GatewayError::UnableToLock); // TODO: Use the correct error / Handle in the function
        };
    } else {
        return Err(GatewayError::NonceNotPresent);
    }
    Ok(())
}

pub fn authenticate_user(
    main_acc: AccountId,
    proxy_acc: Option<AccountId>,
) -> Result<(), GatewayError> {
    // Authentication
    match proxy_acc {
        Some(proxy) => {
            if !polkadex::check_if_proxy_registered(main_acc, proxy)
                .map_err(|e| GatewayError::AccountRegistryError(e))?
            {
                // FIXME: Should this really be an error?
                debug!("Proxy Account is not registered for given Main Account");
                return Err(GatewayError::ProxyNotRegisteredForMainAccount);
            }
        }
        None => {
            if !polkadex::check_if_main_account_registered(main_acc)
                .map_err(|e| GatewayError::AccountRegistryError(e))?
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
    let maker =
        polkadex_orderbook_storage::lock_storage_and_remove_order(&trade.maker_order_uuid)?;
    let taker =
        polkadex_orderbook_storage::lock_storage_and_remove_order(&trade.taker_order_uuid)?;
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
            if order.price.unwrap() == 0 || order.quantity == 0 =>
        {
            Err(GatewayError::QuantityOrPriceZeroInLimitOrder)
        }
        (OrderType::MARKET, OrderSide::BID) if order.price.unwrap() == 0 => {
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
            if let Err(e) = do_asset_exchange(&mut current_order, &mut counter_order, trade_event.amount) {
                error!("Doing asset exchange failed. Error: {:?}", e);
                return Err(GatewayError::UnableToLock); // TODO: Use the correct error
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
            } else {
                let amount_to_unreserve = reserved_amount - trade_event.price;
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
                return Err(GatewayError::UnableToLock); // TODO: Use the correct error
            };
            if counter_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    counter_order.clone(),
                    maker_order_uuid,
                )?;
            }

            if current_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    current_order.clone(),
                    taker_order_uuid,
                )?;
            }
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::ASK) => {
            let reserved_amount = (counter_order.price.unwrap() * counter_order.quantity) / UNIT;
            do_asset_exchange(&mut current_order, &mut counter_order, trade_event.amount)?;
            if counter_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    counter_order.clone(),
                    maker_order_uuid,
                )?;
            } else {
                let amount_to_unreserve = reserved_amount - trade_event.price;
                polkadex_balance_storage::lock_storage_unreserve_balance(
                    &counter_order.user_uid,
                    counter_order.market_id.quote,
                    amount_to_unreserve,
                )?;
            }

            if current_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    current_order.clone(),
                    taker_order_uuid,
                )?;
            }
            Ok(())
        }

        (OrderType::MARKET, OrderSide::ASK) => {
            let reserved_amount = (counter_order.price.unwrap() * counter_order.quantity) / UNIT;
            do_asset_exchange_market(&mut current_order, &mut counter_order)?;
            if counter_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    counter_order.clone(),
                    maker_order_uuid,
                )?;
            } else {
                let amount_to_unreserve = reserved_amount - trade_event.price;
                polkadex_balance_storage::lock_storage_unreserve_balance(
                    &counter_order.user_uid,
                    counter_order.market_id.quote,
                    amount_to_unreserve,
                )?;
            }

            if current_order.quantity > 0 {
                polkadex_orderbook_storage::lock_storage_and_add_order(
                    current_order.clone(),
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
    _expected_trade_amount: u128,
) -> Result<(), GatewayError> {
    match (current_order.order_type, current_order.side) {
        (OrderType::LIMIT, OrderSide::BID) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = ((counter_order.price.unwrap() as f64)
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
            counter_order.quantity = counter_order.quantity - current_order.quantity;
            current_order.quantity = 0;
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::BID) if current_order.quantity > counter_order.quantity => {
            let trade_amount = ((counter_order.price.unwrap() as f64)
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
            current_order.quantity = current_order.quantity - counter_order.quantity;
            counter_order.quantity = 0;
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::ASK) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = ((current_order.price.unwrap() as f64)
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
            counter_order.quantity = counter_order.quantity - current_order.quantity;
            current_order.quantity = 0;
            Ok(())
        }

        (OrderType::LIMIT, OrderSide::ASK) if current_order.quantity > counter_order.quantity => {
            let trade_amount = ((current_order.price.unwrap() as f64)
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
            current_order.quantity = current_order.quantity - counter_order.quantity;
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
                (current_order.price.unwrap() / counter_order.price.unwrap()) * UNIT;
            if current_order_quantity <= counter_order.quantity {
                transfer_asset(
                    &current_order.market_id.quote,
                    current_order.price.unwrap(),
                    &current_order.user_uid,
                    &counter_order.user_uid,
                )?;
                transfer_asset(
                    &current_order.market_id.base,
                    current_order_quantity,
                    &counter_order.user_uid,
                    &current_order.user_uid,
                )?;
                counter_order.quantity = counter_order.quantity - current_order_quantity;
                current_order.price = Some(0);
            } else {
                let trade_amount = (counter_order.price.unwrap() * counter_order.quantity) / UNIT;
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
                current_order.price = Some(current_order.price.unwrap() - trade_amount);
            }
            Ok(())
        }
        (OrderType::MARKET, OrderSide::ASK) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = (counter_order.price.unwrap() * current_order.quantity) / UNIT;
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
            counter_order.quantity = counter_order.quantity - current_order.quantity;
            current_order.quantity = 0;
            Ok(())
        }
        (OrderType::MARKET, OrderSide::ASK) if current_order.quantity > counter_order.quantity => {
            let trade_amount = (counter_order.price.unwrap() * counter_order.quantity) / UNIT;
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
            current_order.quantity = current_order.quantity - counter_order.quantity;
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
    polkadex_balance_storage::lock_storage_unreserve_balance(from, asset_id.clone(), amount)?;
    polkadex_balance_storage::lock_storage_transfer_balance(from, to, asset_id.clone(), amount)?;
    Ok(())
}
