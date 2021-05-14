use codec::{Decode, Encode, Error};
#[cfg(feature = "sgx")]
use sgx_tstd::vec;

#[cfg(feature = "sgx")]
use sgx_tstd::vec::Vec;

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum OrderType {
    LIMIT,
    MARKET,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum OrderSide {
    BID,
    ASK,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum OrderState {
    UNFILLED,
    PARTIAL,
    FILLED,
    CANCELLED,
}

// Create Order
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct Order {
    pub user_uid: Vec<u8>,
    pub market_id: Vec<u8>,
    pub market_type: Vec<u8>,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: u128,
    pub price: Option<u128>,
}

// SignedOrder is used by enclave to store in Orderbook Mirror
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct SignedOrder {
    pub order_id: Vec<u8>,
    pub order: Order,
    pub signature: Vec<u8>, // FIXME: Replace with enclave's signature here
}
impl Default for SignedOrder{
    fn default() -> Self {
        SignedOrder {
            order_id: vec![],
            order: Order {
                user_uid: vec![],
                market_id: vec![],
                market_type: vec![],
                order_type: OrderType::LIMIT,
                side: OrderSide::BID,
                quantity: 0,
                price: None,
            },
            signature: vec![],
        }
    }
}
impl SignedOrder {
    pub fn from_vec(mut k: &[u8]) -> Result<SignedOrder, Error> {
        SignedOrder::decode(&mut k)
    }
}
#[derive(Debug)]
pub struct CreateOrderResponse {
    pub(crate) order_uid: Vec<u8>,
}

// Cancel Orders
#[derive(Debug)]
pub struct CancelOrder {
    user_uid: Vec<u8>,
    market_id: Vec<u8>,
    order_id: Vec<Vec<u8>>,
}

// Deposit Funds
#[derive(Debug)]
pub struct DepositFund {
    user_uid: Vec<u8>,
    currency_id: Vec<u8>,
    amount: u128,
    tx_id: Option<Vec<u8>>,
}

// Withdraw Funds
#[derive(Debug)]
pub struct WithdrawFund {
    user_id: Vec<u8>,
    currency_id: Vec<u8>,
    amount: u128,
    tx_id: Option<Vec<u8>>,
}

// Error
#[derive(Debug)]
pub struct ErrorMessage {
    message: Vec<u8>,
}

// Status Response
#[derive(Debug)]
pub struct Response {
    pub(crate) code: usize,
}

// Order Update Events
#[derive(Debug)]
pub struct OrderUpdate {
    market_id: Vec<u8>,
    order_id: Vec<u8>,
    unique_order_id: Vec<u8>, // Why is there two order ids??
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
    timestamp: Vec<u8>,
}

// Trade Events
#[derive(Debug)]
pub struct TradeEvent {
    market_id: Vec<u8>,
    trade_id: Vec<u8>,
    price: u128,
    amount: u128,
    funds: u128, // price*amount
    maker_order_id: Vec<u8>,
    maker_order_uuid: Vec<u8>,
    taker_order_id: Vec<u8>,
    taker_order_uuid: Vec<u8>,
    maker_side: OrderSide,
    timestamp: Vec<u8>,
}
