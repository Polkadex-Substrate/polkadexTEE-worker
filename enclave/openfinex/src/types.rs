use serde::{Deserialize, Serialize};
use sgx_tstd::string::String;
use sgx_tstd::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderType {
    LIMIT,
    MARKET,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderSide {
    BID,
    ASK,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderState {
    UNFILLED,
    PARTIAL,
    FILLED,
    CANCELLED,
}
// Create Order
#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub user_uid: String,
    pub market_id: String,
    pub market_type: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: u128,
    pub price: Option<u128>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub(crate) order_uid: String,
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
    amount: u128,
    tx_id: Option<String>,
}

// Withdraw Funds
#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawFund {
    user_id: String,
    currency_id: String,
    amount: u128,
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
    pub(crate) code: usize,
}

// Order Update Events
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderUpdate {
    market_id: String,
    order_id: String,
    unique_order_id: String, // Why is there two order ids??
    side: OrderSide,
    kind: OrderSide,
    state: OrderState,
    order_type: OrderType,
    price: u128,
    avg_price: u128,
    current_volume: u128,
    original_volume: u128,
    executed_volume: u128,
    trade_count_order: u128,
    timestamp: String,
}

// Trade Events
#[derive(Debug, Serialize, Deserialize)]
pub struct TradeEvent {
    market_id: String,
    trade_id: String,
    price: u128,
    amount: u128,
    funds: u128, // price*amount
    maker_order_id: String,
    maker_order_uuid: String,
    taker_order_id: String,
    taker_order_uuid: String,
    maker_side: OrderSide,
    timestamp: String,
}
