use polkadex_primitives::openfinex::{OrderUpdate, TradeEvent};
use sgx_types::SgxResult;

// Handle Trade event
/// Assumes that authentication of tradeevent sender happens before
/// this function is called
pub fn handle_trade_event(trade: TradeEvent) -> SgxResult<()> {
    // TODO: Check both orders in Orderbook DB
    // TODO: Check the match methematically
    // TODO: Mutate Balances
    // TODO: Update/Delete Orders from Orderbook DB
    Ok(())
}

// Handle Order update event
pub fn handle_order_update_event(order_update: OrderUpdate) -> SgxResult<()> {
    Ok(())
}
