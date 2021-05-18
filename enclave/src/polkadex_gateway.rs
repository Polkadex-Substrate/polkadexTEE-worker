use log::*;
use polkadex_sgx_primitives::AccountId;
use polkadex_sgx_primitives::types::{Order, OrderSide, OrderType, OrderUUID, TradeEvent};
use sgx_types::{sgx_status_t, SgxResult};

use crate::polkadex;
use crate::polkadex_balance_storage;
use crate::polkadex_orderbook_storage;

/// Place order function does the following
/// 1. authenticate
/// 2. mutate balances (reserve amount offered in order)
/// 3. store_order (async)
/// 4. send order to OpenFinex API
/// 5. report OpenFinex API result to sender
pub fn place_order(main_account: AccountId, proxy_acc: Option<AccountId>, order: Order) -> SgxResult<()> {
    // Authentication
    authenticate_user(main_account.clone(), proxy_acc)?;
    // Mutate Balances
    match order.order_type {
        OrderType::LIMIT => {
            if let Some(price) = order.price {
                match order.side {
                    OrderSide::BID => {
                        // TODO: Is it safe to use saturating_mul here?
                        let amount = price.saturating_mul(order.quantity);
                        polkadex_balance_storage::reserve_balance(&main_account, order.market_id.quote, amount)?;
                    }
                    OrderSide::ASK => {
                        // TODO: Is it safe to use saturating_mul here?
                        polkadex_balance_storage::reserve_balance(&main_account, order.market_id.base, order.quantity)?;
                    }
                }
            } else {
                error!("Price not given for a limit order");
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED); // TODO: It would be better if we can return a error response for RPC to handle
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
                    return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                }
                OrderSide::ASK => {
                    // TODO: Is it safe to use saturating_mul here?
                    polkadex_balance_storage::reserve_balance(&main_account, order.market_id.base, order.quantity)?;
                }
            }
        }
        OrderType::FillOrKill | OrderType::PostOnly => {
            error!("OrderType is not implemented");
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
    }
    // Store the order
    // TODO: we need OrderUUID for storing the order, it is given by Openfinex but we need OrderUUID to
    // TODO: store the order in the orderbook storage
    // polkadex_orderbook_storage::OrderbookStorage::add_order();
    // TODO: Send order to Openfinex for inclusion
    Ok(())
}

/// Place order function does the following
/// 1. authenticate
/// 2. send cancel_order to OpenFinex API
/// 3. remove order from orderbook mirror
/// 4. free reserved balance for the remainder of the order (in case of partial execution)
/// 5. report result to sender
pub fn cancel_order(main_account: AccountId, proxy_acc: Option<AccountId>, order_uuid: OrderUUID) -> SgxResult<()> {
    // Authenticate
    authenticate_user(main_account.clone()  , proxy_acc)?;
    // TODO: Send cancel order to Openfinex API
    // Mutate Balances
    match polkadex_orderbook_storage::remove_order(&order_uuid)? {
        Some(cancelled_order) => {
            match cancelled_order.order_type {
                OrderType::LIMIT => {
                    if let Some(price) = cancelled_order.price {
                        match cancelled_order.side {
                            OrderSide::BID => {
                                // TODO: Is it safe to use saturating_mul here?
                                let amount = price.saturating_mul(cancelled_order.quantity);
                                polkadex_balance_storage::unreserve_balance(main_account.clone(), cancelled_order.market_id.quote, amount)?;
                            }
                            OrderSide::ASK => {
                                // TODO: Is it safe to use saturating_mul here?
                                polkadex_balance_storage::unreserve_balance(main_account.clone(), cancelled_order.market_id.base, cancelled_order.quantity)?;
                            }
                        }
                    } else {
                        error!("Unable to find price for limit order");
                        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                    }
                }
                OrderType::MARKET => {}
                OrderType::FillOrKill | OrderType::PostOnly => {
                    error!("OrderType is not implemented");
                    return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                }
            }
        }
        None => {
            // TODO: Is this error handling correct? Since,
            // TODO: we already sent the request to cancel to Openfinex but
            // TODO: order was not found in Enclave's Orderbook Storage
            error!("Unable to find order for given order_uuid");
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
    }
    Ok(())
}


pub fn authenticate_user(main_acc: AccountId, proxy_acc: Option<AccountId>) -> SgxResult<()> {
    // Authentication
    match proxy_acc {
        Some(proxy) => {
            if !polkadex::check_if_proxy_registered(main_acc, proxy)? {
                error!("Proxy Account is not registered for given Main Account");
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED); // TODO: It would be better if we can return a error response for RPC to handle
            }
        }
        None => {
            if !polkadex::check_if_main_account_registered(main_acc)? {
                error!("Main Account is not registered");
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED); // TODO: It would be better if we can return a error response for RPC to handle
            }
        }
    }
    Ok(())
}