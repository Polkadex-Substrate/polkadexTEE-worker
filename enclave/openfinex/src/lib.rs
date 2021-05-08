#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

use sgx_tstd::string::String;
use sgx_types::{sgx_status_t, SgxResult};

use crate::types::{
    CancelOrder, CreateOrder, CreateOrderResponse, OrderUpdate, Response, TradeEvent,
};

mod tests;
mod tlsclient;
mod types;

pub struct OpenFinexClient;
// Create a WS Client to OpenFinex
pub fn subscribe_to_openfinex_events(address: &str) -> SgxResult<OpenFinexClient> {
    Ok(OpenFinexClient)
}

// Forwards the Create Order placed via RPC to OpenFinex
pub fn send_place_order_req_to_openfinex(
    api: OpenFinexClient,
    order: CreateOrder,
) -> SgxResult<CreateOrderResponse> {
    Ok(CreateOrderResponse {
        order_uid: String::from("sample"),
    })
}

// Forwards the Cancel Order placed via RPC to OpenFinex
pub fn send_cancel_order_req_to_openfinex(
    api: OpenFinexClient,
    order: CancelOrder,
) -> SgxResult<Response> {
    Ok(Response { code: 0 })
}

// Handle Trade event
pub fn handle_trade_event(trade: TradeEvent) -> SgxResult<()> {
    Ok(())
}

// Handle Order update event
pub fn handle_order_update_event(order_update: OrderUpdate) -> SgxResult<()> {
    Ok(())
}
