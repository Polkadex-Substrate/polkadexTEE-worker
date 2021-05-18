use codec::{Decode, Encode};
use log::*;
use polkadex_sgx_primitives::AccountId;
use polkadex_sgx_primitives::types::{Order, OrderSide, OrderType, OrderUUID, TradeEvent};
use sgx_types::{sgx_status_t, SgxResult};
use fixed::FixedU128;
use crate::polkadex;
use crate::polkadex_balance_storage;
use crate::polkadex_orderbook_storage;

#[derive(Encode, Decode, Debug)]
pub enum GatewayError {
    /// Price for limit Order not found
    LimitOrderPriceNotFound,
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
    UndefinedBehaviour
}

/// Place order function does the following
/// 1. authenticate
/// 2. mutate balances (reserve amount offered in order)
/// 3. store_order (async)
/// 4. send order to OpenFinex API
/// 5. report OpenFinex API result to sender
pub fn place_order(main_account: AccountId, proxy_acc: Option<AccountId>, order: Order) -> Result<(), GatewayError> {
    // Authentication
    authenticate_user(main_account.clone(), proxy_acc)?;
    // Mutate Balances
    match order.order_type {
        OrderType::LIMIT => {
            if let Some(price) = order.price {
                match order.side {
                    OrderSide::BID => {
                        // let amount = price.saturating_mul(order.quantity);
                        let amount = FixedU128::from(price).saturating_mul(FixedU128::from(order.quantity)).saturating_to_num::<u128>();
                        match polkadex_balance_storage::reserve_balance(&main_account, order.market_id.quote, amount) {
                            Ok(()) => {},
                            Err(e) => return Err(GatewayError::FailedToReserveBalance)
                        };
                    }
                    OrderSide::ASK => {
                        match polkadex_balance_storage::reserve_balance(&main_account, order.market_id.base, order.quantity) {
                            Ok(()) => { } ,
                            Err(e) => return Err(GatewayError::FailedToReserveBalance)
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
                OrderSide::BID => {
                    // TODO: Is it safe to use saturating_mul here?
                    // TODO: How do we reserve trade amount for Market Buy since it is not possible to define price before
                    // TODO: order has been matched in the orderbook.
                    // let amount = price.saturating_mul(order.quantity);
                    // polkadex_balance_storage::reserve_balance(&main_account, order.market_id.quote, amount)?;
                    error!("Market Buy is not implemented");
                    return Err(GatewayError::NotImplementedYet);
                }
                OrderSide::ASK => {
                    match polkadex_balance_storage::reserve_balance(&main_account, order.market_id.base, order.quantity){
                        Ok(()) => {},
                        Err(e) => return Err(GatewayError::FailedToReserveBalance)
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
    // TODO: we need OrderUUID for storing the order, it is given by Openfinex but we need OrderUUID to
    // TODO: store the order in the orderbook storage
    // polkadex_orderbook_storage::OrderbookStorage::add_order();
    // TODO: Send order async to Openfinex for inclusion
    Ok(())
}

/// Place order function does the following
/// 1. authenticate
/// 2. send cancel_order to OpenFinex API
/// 3. remove order from orderbook mirror
/// 4. free reserved balance for the remainder of the order (in case of partial execution)
/// 5. report result to sender
pub fn cancel_order(main_account: AccountId, proxy_acc: Option<AccountId>, order_uuid: OrderUUID) -> Result<(), GatewayError> {
    // Authenticate
    authenticate_user(main_account.clone(), proxy_acc)?;
    // TODO: Send cancel order to Openfinex API
    // Mutate Balances

    if let Ok(result) = polkadex_orderbook_storage::remove_order(&order_uuid){
        match result {
            Some(cancelled_order) => {
                match cancelled_order.order_type {
                    OrderType::LIMIT => {
                        if let Some(price) = cancelled_order.price {
                            match cancelled_order.side {
                                OrderSide::BID => {
                                    // let amount = price.saturating_mul(cancelled_order.quantity);
                                    let amount = FixedU128::from(price).saturating_mul(FixedU128::from(cancelled_order.quantity)).saturating_to_num::<u128>();
                                    match polkadex_balance_storage::unreserve_balance(main_account.clone(), cancelled_order.market_id.quote, amount){
                                        Ok(()) => { } ,
                                        Err(e) => return Err(GatewayError::FailedToUnReserveBalance)
                                    };
                                }
                                OrderSide::ASK => {
                                    match polkadex_balance_storage::unreserve_balance(main_account.clone(), cancelled_order.market_id.base, cancelled_order.quantity){
                                        Ok(()) => {},
                                        Err(e) => return Err(GatewayError::FailedToUnReserveBalance)
                                    };
                                }
                            }
                        } else {
                            error!("Unable to find price for limit order");
                            return Err(GatewayError::LimitOrderPriceNotFound);
                        }
                    }
                    OrderType::MARKET => {
                        error!("OrderType is not implemented");
                        return Err(GatewayError::UndefinedBehaviour);
                    }
                    OrderType::FillOrKill | OrderType::PostOnly => {
                        error!("OrderType is not implemented");
                        return Err(GatewayError::NotImplementedYet);
                    }
                }
            }
            None => {
                error!("Unable to find order for given order_uuid");
                return Err(GatewayError::OrderNotFound);
            }
        }
    }
    else{
        return Err(GatewayError::UnableToRemoveOrder)
    }
    Ok(())
}


pub fn authenticate_user(main_acc: AccountId, proxy_acc: Option<AccountId>) -> Result<(), GatewayError> {
    // Authentication
    match proxy_acc {
        Some(proxy) => {
            if !polkadex::check_if_proxy_registered(main_acc, proxy).map_err(|_| GatewayError::UndefinedBehaviour)? {
                error!("Proxy Account is not registered for given Main Account");
                return Err(GatewayError::ProxyNotRegisteredForMainAccount);
            }
        }
        None => {
            if !polkadex::check_if_main_account_registered(main_acc).map_err(|_| GatewayError::UndefinedBehaviour)? {
                error!("Main Account is not registered");
                return Err(GatewayError::MainAccountNotRegistered);
            }
        }
    }
    Ok(())
}