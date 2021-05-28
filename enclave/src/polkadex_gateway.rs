use codec::{Decode, Encode};
use frame_support::ensure;
use log::*;
use polkadex_sgx_primitives::types::{Order, OrderSide, OrderType, OrderUUID, TradeEvent, UserId};
use polkadex_sgx_primitives::{AccountId, AssetId, Balance};
use sgx_types::{sgx_status_t, SgxResult};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, SgxMutex, SgxMutexGuard};

use crate::constants::UNIT;
use crate::polkadex;
use crate::polkadex_balance_storage;
use crate::polkadex_orderbook_storage;

#[derive(Encode, Decode, Debug, PartialOrd, PartialEq)]
pub enum GatewayError {
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
}

/// Place order function does the following
/// 1. authenticate
/// 2. mutate balances (reserve amount offered in order)
/// 3. store_order (async)
/// 4. send order to OpenFinex API
/// 5. report OpenFinex API result to sender
pub fn place_order(
    main_account: AccountId,
    proxy_acc: Option<AccountId>,
    order: Order,
) -> Result<OrderUUID, GatewayError> {
    // Authentication
    authenticate_user(main_account.clone(), proxy_acc)?;
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
                        let amount =
                            ((price as f64) * ((order.quantity as f64) / (UNIT as f64))) as u128;
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
    // if let Ok(nonce) = get_finex_nonce_and_increment() {
    //     if let Ok(mutex) = load_create_cache_pointer() {
    //         let mut cache: SgxMutexGuard<HashMap<u128, Order>> = mutex.lock().unwrap();
    //         cache.insert(nonce, order);
    //     } else {
    //         error!("Unable to get new nonce for order");
    //         return Err(GatewayError::UndefinedBehaviour);
    //     }
    // } else {
    //     error!("Unable to get new nonce for order");
    //     return Err(GatewayError::UndefinedBehaviour);
    // }

    let order_uuid: OrderUUID = send_order_to_open_finex(order.clone())?;
    polkadex_orderbook_storage::add_order(order, order_uuid.clone())
        .map_err(|_| GatewayError::UndefinedBehaviour)?; // TODO: Change the error type of add order to GateWay Error.
    Ok(order_uuid)
}

fn send_order_to_open_finex(order: Order) -> Result<OrderUUID, GatewayError> {
    // TODO: Send order to Openfinex for inclusion ( this is a blocking call )
    Ok(OrderUUID::new())
}

fn send_cancel_request_to_openfinex(order_uuid: &OrderUUID) -> Result<(), GatewayError> {
    // TODO: Send cancel order to Openfinex API ( this is a blocking call)
    Ok(())
}

/// Cancel order function does the following
/// 1. authenticate
/// 2. Cache the cancel request
/// 3. send cancel_order to OpenFinex API
pub fn cancel_order(
    main_account: AccountId,
    proxy_acc: Option<AccountId>,
    order_uuid: OrderUUID,
) -> Result<(), GatewayError> {
    // Authenticate
    authenticate_user(main_account.clone(), proxy_acc)?;
    // if let Ok(mutex) = load_cancel_cache_pointer() {
    //     let mut cancel_cache: SgxMutexGuard<HashSet<OrderUUID>> = mutex.lock().unwrap();
    //     cancel_cache.insert(order_uuid);
    //     // Send cancel order to Openfinex API
    // }
    send_cancel_request_to_openfinex(&order_uuid)?;
    let cancelled_order = polkadex_orderbook_storage::remove_order(&order_uuid)?;
    match (cancelled_order.order_type, cancelled_order.side) {
        (OrderType::LIMIT, OrderSide::BID) => {
            let price = cancelled_order
                .price
                .ok_or(GatewayError::LimitOrderPriceNotFound)?;
            let amount =
                ((price as f64) * ((cancelled_order.quantity as f64) / (UNIT as f64))) as u128;
            polkadex_balance_storage::lock_storage_unreserve_balance(
                cancelled_order.user_uid,
                cancelled_order.market_id.quote,
                amount,
            )?;
        }

        (OrderType::LIMIT, OrderSide::ASK) => {
            polkadex_balance_storage::lock_storage_unreserve_balance(
                cancelled_order.user_uid,
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

// /// process_cancel_order does the following
// /// 1. Checks the orderUUID with cancel request cache
// /// 2. Remove order from Orderbook Mirror
// /// 3. Mutate the balances
// pub fn process_cancel_order(order_uuid: OrderUUID) -> Result<(), GatewayError> {
//     if let Ok(mutex) = load_cancel_cache_pointer() {
//         let mut cancel_cache: SgxMutexGuard<HashSet<OrderUUID>> = mutex.lock().unwrap();
//         if !cancel_cache.remove(&order_uuid) {
//             error!("Order Cancel Request not found in Cache");
//             return Err(GatewayError::UnableToCancelOrder);
//         }
//         // Mutate Balances
//         if let Ok(result) = polkadex_orderbook_storage::remove_order(&order_uuid) {
//             match result {
//                 Some(cancelled_order) => {
//                     match cancelled_order.order_type {
//                         OrderType::LIMIT => {
//                             if let Some(price) = cancelled_order.price {
//                                 match cancelled_order.side {
//                                     OrderSide::BID => {
//                                         let amount = ((price as f64) * ((cancelled_order.quantity as f64) / (UNIT as f64))) as u128;
//                                         match polkadex_balance_storage::unreserve_balance(cancelled_order.user_uid, cancelled_order.market_id.quote, amount) {
//                                             Ok(()) => {}
//                                             Err(_) => return Err(GatewayError::FailedToUnReserveBalance)
//                                         };
//                                     }
//                                     OrderSide::ASK => {
//                                         match polkadex_balance_storage::unreserve_balance(cancelled_order.user_uid, cancelled_order.market_id.base, cancelled_order.quantity) {
//                                             Ok(()) => {}
//                                             Err(_) => return Err(GatewayError::FailedToUnReserveBalance)
//                                         };
//                                     }
//                                 }
//                             } else {
//                                 error!("Unable to find price for limit order");
//                                 return Err(GatewayError::LimitOrderPriceNotFound);
//                             }
//                         }
//                         OrderType::MARKET => {
//                             error!("Cancel Order is not applicable for Market Order");
//                             return Err(GatewayError::UndefinedBehaviour);
//                         }
//                         OrderType::FillOrKill | OrderType::PostOnly => {
//                             error!("OrderType is not implemented");
//                             return Err(GatewayError::NotImplementedYet);
//                         }
//                     }
//                 }
//                 None => {
//                     error!("Unable to find order for given order_uuid");
//                     return Err(GatewayError::OrderNotFound);
//                 }
//             }
//         } else {
//             return Err(GatewayError::UnableToRemoveOrder);
//         }
//         return Ok(());
//     }
//     error!("Unable to load the cancel cache pointer");
//     Err(GatewayError::UndefinedBehaviour)
// }

pub fn authenticate_user(
    main_acc: AccountId,
    proxy_acc: Option<AccountId>,
) -> Result<(), GatewayError> {
    // Authentication
    match proxy_acc {
        Some(proxy) => {
            if !polkadex::check_if_proxy_registered(main_acc, proxy)
                .map_err(|_| GatewayError::UndefinedBehaviour)?
            {
                debug!("Proxy Account is not registered for given Main Account");
                return Err(GatewayError::ProxyNotRegisteredForMainAccount);
            }
        }
        None => {
            if !polkadex::check_if_main_account_registered(main_acc)
                .map_err(|_| GatewayError::UndefinedBehaviour)?
            {
                debug!("Main Account is not registered");
                return Err(GatewayError::MainAccountNotRegistered);
            }
        }
    }
    Ok(())
}

// static CREATE_ORDER_NONCE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());
// static CREATE_ORDER_CACHE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());
// static CANCEL_ORDER_CACHE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

// pub fn initialize_polkadex_gateway() {
//     // let nonce: u128 = 0;
//     // let create_nonce_storage_ptr = Arc::new(SgxMutex::<u128>::new(nonce));
//     // let create_nonce_ptr = Arc::into_raw(create_nonce_storage_ptr);
//     // CREATE_ORDER_NONCE.store(create_nonce_ptr as *mut (), Ordering::SeqCst);
//     //
//     // let cancel_cache: HashSet<OrderUUID> = HashSet::new();
//     // let cancel_cache_storage_ptr = Arc::new(SgxMutex::new(cancel_cache));
//     // let cancel_cache_ptr = Arc::into_raw(cancel_cache_storage_ptr);
//     // CANCEL_ORDER_CACHE.store(cancel_cache_ptr as *mut (), Ordering::SeqCst);
//     //
//     // let create_cache: HashMap<u128, Order> = HashMap::new();
//     // let create_cache_storage_ptr = Arc::new(SgxMutex::new(create_cache));
//     // let create_cache_ptr = Arc::into_raw(create_cache_storage_ptr);
//     // CREATE_ORDER_CACHE.store(create_cache_ptr as *mut (), Ordering::SeqCst);
// }
//
// fn load_finex_nonce_pointer() -> SgxResult<&'static SgxMutex<u128>> {
//     let ptr = CREATE_ORDER_NONCE.load(Ordering::SeqCst) as *mut SgxMutex<u128>;
//     if ptr.is_null() {
//         error!("Pointer is Null");
//         return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
//     } else {
//         Ok(unsafe { &*ptr })
//     }
// }
//
// fn load_cancel_cache_pointer() -> SgxResult<&'static SgxMutex<HashSet<OrderUUID>>> {
//     let ptr = CANCEL_ORDER_CACHE.load(Ordering::SeqCst) as *mut SgxMutex<HashSet<OrderUUID>>;
//     if ptr.is_null() {
//         error!("Pointer is Null");
//         return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
//     } else {
//         Ok(unsafe { &*ptr })
//     }
// }
//
// fn load_create_cache_pointer() -> SgxResult<&'static SgxMutex<HashMap<u128, Order>>> {
//     let ptr = CREATE_ORDER_CACHE.load(Ordering::SeqCst) as *mut SgxMutex<HashMap<u128, Order>>;
//     if ptr.is_null() {
//         error!("Pointer is Null");
//         return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
//     } else {
//         Ok(unsafe { &*ptr })
//     }
// }
//
// fn get_finex_nonce_and_increment() -> SgxResult<u128> {
//     let mutex = load_nonce_pointer()?;
//     let mut nonce: SgxMutexGuard<u128> = mutex.lock().unwrap();
//     let current_nonce = nonce.clone();
//     nonce.saturating_add(1);
//     Ok(current_nonce)
// }
//
//
// pub fn remove_order_from_cache_and_store_in_ordermirror(nonce: u128, order_uuid: OrderUUID) -> SgxResult<()> {
//     let mutex = load_create_cache_pointer()?;
//     let cache: SgxMutexGuard<HashMap<u128, Order>> = mutex.lock().unwrap();
//     if let Some(order) = cache.get(&nonce) {
//         polkadex_orderbook_storage::add_order(order.clone(), order_uuid)?;
//          TODO: Remove order from cache
//     } else {
//         error!("Unable to find order for the given nonce");
//         return Err(Default::default());
//     }
//     Ok(())
// }

pub fn settle_trade(trade: TradeEvent) -> Result<(), GatewayError> {
    // Check if both orders exist and get them
    let maker = polkadex_orderbook_storage::remove_order(&trade.maker_order_uuid)?;
    let mut taker = polkadex_orderbook_storage::remove_order(&trade.taker_order_uuid)?;
    ensure!(
        (maker.market_id == taker.market_id) & (maker.market_id == trade.market_id),
        GatewayError::MarketIdMismatch
    );
    ensure!(
        maker.side == trade.maker_side,
        GatewayError::MakerSideMismatch
    );
    // Derive buyer and seller from maker and taker

    ensure!(
        taker.price.unwrap() > maker.price.unwrap(),
        GatewayError::MakerSideMismatch
    );
    basic_order_checks(&maker)?;
    basic_order_checks(&taker)?;
    consume_order(&mut taker, &mut maker)?;

    Ok(())
}

pub fn basic_order_checks(order: &Order) -> Result<(), GatewayError> {
    match (order.order_type, order.side) {
        (OrderType::LIMIT, OrderSide::BID) | (OrderType::LIMIT, OrderSide::ASK)
            if order.price <= 0 || order.quantity <= 0 =>
        {
            Err(GatewayError::LimitOrderPriceNotFound)
        }
        (OrderType::MARKET, OrderSide::BID) if order.price <= 0 => {
            Err(GatewayError::LimitOrderPriceNotFound)
        }
        (OrderType::MARKET, OrderSide::ASK) if order.quantity <= 0 => {
            Err(GatewayError::LimitOrderPriceNotFound)
        }
        _ => Ok(()),
    }
}

pub fn consume_order(current_order: &mut Order, counter_order: &mut Order) {
    match (*current_order.order_type, *current_order.side) {
        (OrderType::LIMIT, OrderSide::BID) => {
            do_asset_exchange(current_order, counter_order);
            if counter_order.quantity > 0 {
                //TODO Insert counter order agian -- Also look into else
            }
        }

        (OrderType::MARKET, OrderSide::BID) => {
            do_asset_exchange_market(current_order, counter_order);
            if counter_order.quantity > 0 {
                //TODO counter order again -- Also look into else
            }
        }

        (OrderType::LIMIT, OrderSide::ASK) => {
            do_asset_exchange(current_order, counter_order);
            if counter_order.quantity > 0 {
                //TODO counter order again -- Also look into else
            }
        }

        (OrderType::MARKET, OrderSide::ASK) => {
            do_asset_exchange_market(current_order, counter_order);
            if counter_order.quantity > 0 {
                //TODO counter order again -- Also look into else
            }
        }
    }
}

pub fn do_asset_exchange(current_order: &mut Order, counter_order: &mut Order) {
    match (*current_order.order_type, *current_order.side) {
        (OrderType::LIMIT, OrderSide::BID) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = counter_order.price * current_order.quantity; //FIXME : Multiplication
            transfer_asset_current(
                current_order.market_id.base,
                trade_amount,
                current_order.user_uid,
                counter_order.user_uid,
            );
            transfer_asset(
                current_order.market_id.quote,
                current_order.quantity,
                *counter_order.user_uid,
                *current_order.user_uid,
            );
            counter_order.quantity = counter_order.quantity - current_order.quantity;
            current_order.quantity = 0; //TODO :- Should we remove order here
        }

        (OrderType::LIMIT, OrderSide::BID) if current_order.quantity > counter_order.quantity => {
            let trade_amount = counter_order.price * counter_order.quantity;
            transfer_asset_current(
                current_order.market_id.base,
                trade_amount,
                current_order.user_uid,
                counter_order.user_uid,
            );
            transfer_asset(
                current_order.market_id.quote,
                counter_order.quantity,
                counter_order.user_uid,
                current_order.user_uid,
            );
            current_order.quantity = current_order.quantity - counter_order.quantity;
            counter_order.quantity = 0;
            //TODO Should we remove maker order here
        }

        (OrderType::LIMIT, OrderSide::ASK) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = counter_order.price * current_order.quantity;
            transfer_asset(
                current_order.market_id.base,
                trade_amount,
                counter_order.user_uid,
                current_order.user_uid,
            );
            transfer_asset_current(
                current_order.market_id.quote,
                current_order.quantity,
                current_order.user_uid,
                counter_order.user_uid,
            );
            counter_order.quantity = counter_order.quantity - current_order.quantity;
            current_order.quantity = 0;
        }

        (OrderType::LIMIT, OrderSide::ASK) if current_order.quantity > counter_order.quantity => {
            let trade_amount = counter_order.price * current_order.quantity;
            transfer_asset(
                current_order.market_id.base,
                trade_amount,
                counter_order.user_uid,
                current_order.user_uid,
            );
            transfer_asset_current(
                current_order.market_id.quote,
                counter_order.quantity,
                current_order.user_uid,
                counter_order.user_uid,
            );
            current_order.quantity = current_order.quantity - counter_order.quantity;
            counter_order.quantity = 0;
        }
        _ => {}
    }
}

pub fn do_asset_exchange_market(current_order: &mut Order, counter_order: &mut Order) {
    match (current_order.order_type, current_order.side) {
        (OrderType::MARKET, OrderSide::BID) => {
            let current_order_quantity = current_order.price / counter_order.price;
            if current_order_quantity <= counter_order.quantity {
                transfer_asset_current(
                    current_order.market_id.base,
                    current_order.price,
                    current_order.user_uid,
                    counter_order.user_uid,
                );
                transfer_asset(
                    current_order.market_id.quote,
                    current_order_quantity,
                    counter_order.user_uid,
                    current_order.user_uid,
                );
                counter_order.quantity = counter_order.quantity - current_order_quantity;
                current_order.price = 0;
            } else {
                let trade_amount = counter_order.price * counter_order.quantity;
                transfer_asset_current(
                    current_order.market_id.base,
                    trade_amount,
                    current_order.user_uid,
                    counter_order.user_uid,
                );
                transfer_asset(
                    current_order.market_id.quote,
                    counter_order.quantity,
                    counter_order.user_uid,
                    current_order.user_uid,
                );
                counter_order.quantity = 0;
                current_order.price = current_order.price - trade_amount;
            }
        }
        (OrderType::MARKET, OrderSide::ASK) if current_order.quantity <= counter_order.quantity => {
            let trade_amount = counter_order.price * current_order.quantity;
            transfer_asset(
                current_order.market_id.base,
                trade_amount,
                counter_order.user_uid,
                current_order.user_uid,
            );
            transfer_asset_current(
                current_order.market_id.quote,
                current_order.quantity,
                current_order.user_uid,
                counter_order.user_uid,
            );
            counter_order.quantity = counter_order.quantity - current_order.quantity;
            current_order.quantity = 0;
        }
        (OrderType::MARKET, OrderSide::ASK) if current_order.quantity > counter_order.quantity => {
            let trade_amount = counter_order.price * counter_order.quantity;
            transfer_asset(
                current_order.market_id.base,
                trade_amount,
                counter_order.user_uid,
                current_order.user_uid,
            );
            transfer_asset_current(
                current_order.market_id.quote,
                counter_order.quantity,
                current_order.user_uid,
                counter_order.user_uid,
            );
            current_order.quantity = current_order.quantity - counter_order.quantity;
            counter_order.quantity = 0;
        }
    }
}

pub fn transfer_asset(asset_id: AssetId, amount: u128, from: UserId, to: UserId) {

    // First unreserve
    // Transfer amount
}

pub fn transfer_asset_current(asset_id: AssetId, amount: u128, from: UserId, to: UserId) {
    // Direct transfer
}
