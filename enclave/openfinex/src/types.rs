use serde::{Deserialize, Serialize};
use sgx_tstd::string::String;
use sgx_tstd::vec::Vec;
// Create Order
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrder {
    user_uid: String,
    market_id: String,
    market_type: String,
    order_type: String,
    side: String,
    quantity: String,
    price: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    order_uid: String,
}

// Cancel Orders
#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrder {
    user_uid: String,
    market_id: String,
    order_id: Vec<String>,
}

// Deposit Funds
#[derive(Debug, Serialize, Deserialize)]
pub struct DepositFund {
    user_uid: String,
    currency_id: String,
    amount: String,
    tx_id: Option<String>,
}

// Withdraw Funds
#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawFund {
    user_id: String,
    currency_id: String,
    amount: String,
    tx_id: Option<String>,
}

// Error
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMessage {
    message: String,
}

// Status Response
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    code: usize,
}

// Order Update Events
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderUpdate {
    market_id: String,
    order_id: String,
    unique_order_id: String, // Why is there two order ids??
    side: String,
    kind: String,
    state: String,
    order_type: String,
    price: String,
    avg_price: String,
    current_volume: String,
    original_volume: String,
    executed_volume: String,
    trade_count_order: String,
    timestamp: String,
}

// Trade Events
#[derive(Debug, Serialize, Deserialize)]
pub struct TradeEvent {
    market_id: String,
    trade_id: String,
    price: String,
    amount: String,
    funds: String, // price*amount
    maker_order_id: String,
    maker_order_uuid: String,
    taker_order_id: String,
    taker_order_uuid: String,
    maker_side: String,
    timestamp: String,
}
