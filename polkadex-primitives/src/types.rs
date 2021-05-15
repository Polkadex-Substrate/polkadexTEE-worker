use codec::{Decode, Encode, Error};
#[cfg(feature = "sgx")]
use sgx_tstd::vec;
#[cfg(feature = "sgx")]
use sgx_tstd::vec::Vec;

/// User UID or nickname to identify the user (Wallet Address in our case)
pub type UserId = Vec<u8>;
/// Unique order ID
pub type OrderId = Vec<u8>;
/// Unique order uuid
pub type OrderUUID = Vec<u8>;
/// Market identifier for order ex: "btcusd"
pub type MarketId = Vec<u8>;
/// Unique trade ID
pub type TradeId = Vec<u8>;
/// Date type for Price and Volume
pub type PriceAndQuantityType = u128;
/// Market type ex: "trusted"
pub type MarketType = Vec<u8>;
/// Currency identifier
pub type CurrencyId = Vec<u8>;

/// The different Order Types
/// - market: "m"
/// - limit: "l"
/// - Post only (Must not fill at all or is canceled): "p"
/// - Fill or kill (Must fully match at a given price or iscanceled): "f"
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum OrderType {
    LIMIT,
    MARKET,
    PostOnly,
    FillOrKill,
}

/// Used to specify order side, "buy" or "sell"
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
    pub user_uid: UserId,
    pub market_id: MarketId,
    pub market_type: MarketType,
    pub order_type: OrderType,
    pub side: OrderSide,
    // An amount that placed within the order
    pub quantity: PriceAndQuantityType,
    // Main (limit) price of the order (optional)
    pub price: Option<PriceAndQuantityType>,
}

// SignedOrder is used by enclave to store in Orderbook Mirror
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct SignedOrder {
    pub order_id: OrderId,
    pub order: Order,
    pub signature: Vec<u8>, // FIXME: Replace with enclave's signature here
}

impl Default for SignedOrder {
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

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateOrderResponse {
    pub(crate) order_uid: OrderUUID,
}

// Cancel Orders
#[derive(Debug, Clone, Encode, Decode)]
pub struct CancelOrder {
    // User UID or nickname to identify the user
    user_uid: UserId,
    // Market identifier for order ex: "btcusd"
    market_id: MarketId,
    // List of order IDs or UUIDs to cancel
    order_id: Vec<OrderId>,
}

// Deposit Funds
#[derive(Debug, Clone, Encode, Decode)]
pub struct DepositFund {
    // User UID or nickname to identify the user
    user_uid: UserId,
    // Currency identifier of the deposit
    currency_id: CurrencyId,
    // Amount to deposit
    amount: PriceAndQuantityType,
    // Transaction ID (optional)
    tx_id: Option<Vec<u8>>,
}

// Withdraw Funds
#[derive(Debug, Clone, Encode, Decode)]
pub struct WithdrawFund {
    // User UID or nickname to identify the user
    user_id: UserId,
    // Currency identifier of the deposit
    currency_id: CurrencyId,
    // Amount to deposit
    amount: PriceAndQuantityType,
    // Transaction ID (optional)
    tx_id: Option<Vec<u8>>,
}

// Error
#[derive(Debug, Clone, Encode, Decode)]
pub struct ErrorMessage {
    message: Vec<u8>,
}

// Status Response
#[derive(Debug, Clone, Encode, Decode)]
pub struct Response {
    pub(crate) code: u32,
}

// Order Update Events
#[derive(Debug, Clone, Encode, Decode)]
pub struct OrderUpdate {
    // Market unique identifier
    market_id: MarketId,
    // Unique order ID
    order_id: OrderId,
    // Unique order uuid
    unique_order_id: OrderUUID,
    // Why is there two order ids??
    // "buy" or "sell"
    side: OrderSide,
    // "bid" or "ask"
    kind: OrderSide,
    // Current state of the order
    state: OrderState,
    // Order type
    order_type: OrderType,
    // Order price
    price: PriceAndQuantityType,
    // Average execution price
    avg_price: PriceAndQuantityType,
    // Order volume
    current_volume: PriceAndQuantityType,
    // Origin Volume
    original_volume: PriceAndQuantityType,
    // Executed Volume
    executed_volume: PriceAndQuantityType,
    // Trade Count
    trade_count_order: PriceAndQuantityType,
    // Order Creation Timestamp
    timestamp: Vec<u8>,
}

// Trade Events
#[derive(Debug, Clone, Encode, Decode)]
pub struct TradeEvent {
    // Market Unique Identifier
    market_id: MarketId,
    // Unique Trade ID
    trade_id: TradeId,
    // Trade execution price
    price: PriceAndQuantityType,
    // Trade execution amount
    amount: PriceAndQuantityType,
    // Trade Funds (amount*price)
    funds: PriceAndQuantityType,
    // Maker's trade Order Id
    maker_order_id: OrderId,
    // Maker's trade Order UUID
    maker_order_uuid: OrderUUID,
    // Taker's trade Order Id
    taker_order_id: OrderId,
    // Taker's trade Order UUID
    taker_order_uuid: OrderUUID,
    // Maker Order Side
    maker_side: OrderSide,
    // Trade Timestamp
    timestamp: Vec<u8>,
}
