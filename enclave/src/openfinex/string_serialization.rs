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
use crate::openfinex::openfinex_api::OpenFinexApiError;
use crate::polkadex_cache::cache_api::CacheProvider;
use crate::polkadex_cache::market_cache::MarketCache;
use crate::ss58check::account_id_to_ss58check;
use alloc::{string::String, string::ToString, sync::Arc};
use codec::Decode;
use polkadex_sgx_primitives::types::{
    MarketId, MarketType, OrderSide, OrderState, OrderType, UserId,
};
use polkadex_sgx_primitives::AssetId;
use sp_core::H160;

pub trait OpenFinexResponseDeserializer {
    fn string_to_market_id(&self, market_id_str: &str) -> Result<MarketId, String>;

    fn string_to_order_type(&self, order_type_str: &str) -> Result<OrderType, String>;

    fn string_to_order_side(&self, order_side_str: &str) -> Result<OrderSide, String>;

    fn string_to_order_state(&self, order_state_str: &str) -> Result<OrderState, String>;

    fn string_to_asset_id(&self, asset_id_str: &str) -> Result<AssetId, String>;
}

pub struct ResponseDeserializerImpl {
    market_cache_provider: Arc<dyn CacheProvider<MarketCache>>,
}

impl ResponseDeserializerImpl {
    pub fn new(
        market_cache_provider: Arc<dyn CacheProvider<MarketCache>>,
    ) -> ResponseDeserializerImpl {
        ResponseDeserializerImpl {
            market_cache_provider,
        }
    }
}

impl OpenFinexResponseDeserializer for ResponseDeserializerImpl {
    fn string_to_market_id(&self, market_id_str: &str) -> Result<MarketId, String> {
        let mutex = self
            .market_cache_provider
            .load()
            .map_err(|_| "Failed to load market cache".to_string())?;

        let cache = mutex
            .lock()
            .map_err(|e| format!("Could not acquire lock on market cache: {}", e))?;

        string_to_market_id(market_id_str, &cache)
    }

    fn string_to_order_type(&self, order_type_str: &str) -> Result<OrderType, String> {
        string_to_order_type(order_type_str)
    }

    fn string_to_order_side(&self, order_side_str: &str) -> Result<OrderSide, String> {
        string_to_order_side(order_side_str)
    }

    fn string_to_order_state(&self, order_state_str: &str) -> Result<OrderState, String> {
        string_to_order_state(order_state_str)
    }

    fn string_to_asset_id(&self, asset_id_str: &str) -> Result<AssetId, String> {
        asset_id_mapping::string_to_asset_id(asset_id_str)
    }
}

pub fn user_id_to_request_string(user_id: &UserId) -> String {
    account_id_to_ss58check(user_id)
}

pub fn market_id_to_request_string(market_id: MarketId) -> String {
    format!(
        "{}{}",
        asset_id_mapping::asset_id_to_string(market_id.base),
        asset_id_mapping::asset_id_to_string(market_id.quote)
    )
}

fn string_to_market_id(
    market_id_str: &str,
    market_cache: &MarketCache,
) -> Result<MarketId, String> {
    let market = market_cache.get_market(market_id_str).ok_or_else(|| {
        format!(
            "Could not find a market object in cache for id '{}'",
            market_id_str
        )
    })?;

    let base_asset = asset_id_mapping::string_to_asset_id(&market.base_unit)?;
    let quote_asset = asset_id_mapping::string_to_asset_id(&market.quote_unit)?;

    Ok(MarketId {
        base: base_asset,
        quote: quote_asset,
    })
}

pub fn market_type_to_request_string(market_type: MarketType) -> Result<String, OpenFinexApiError> {
    String::from_utf8(market_type).map_err(|e| OpenFinexApiError::SerializationError(e.to_string()))
}

pub fn order_uuid_to_request_string(order_uuid: MarketType) -> Result<String, OpenFinexApiError> {
    //String::from_utf8(order_uuid).map_err(|e| OpenFinexApiError::SerializationError(e.to_string()))
    String::decode(&mut order_uuid.as_slice())
        .map_err(|e| OpenFinexApiError::SerializationError(e.to_string()))
}

const MARKET_ORDER_TYPE_STR: &str = "m";
const LIMIT_ORDER_TYPE_STR: &str = "l";
const POSTONLY_ORDER_TYPE_STR: &str = "p";
const FILLORKILL_ORDER_TYPE_STR: &str = "f";

pub fn order_type_to_request_string(order_type: OrderType) -> String {
    match order_type {
        OrderType::MARKET => MARKET_ORDER_TYPE_STR.to_string(),
        OrderType::LIMIT => LIMIT_ORDER_TYPE_STR.to_string(),
        OrderType::PostOnly => POSTONLY_ORDER_TYPE_STR.to_string(),
        OrderType::FillOrKill => FILLORKILL_ORDER_TYPE_STR.to_string(),
    }
}

fn string_to_order_type(order_type_str: &str) -> Result<OrderType, String> {
    match order_type_str {
        MARKET_ORDER_TYPE_STR => Ok(OrderType::MARKET),
        LIMIT_ORDER_TYPE_STR => Ok(OrderType::LIMIT),
        POSTONLY_ORDER_TYPE_STR => Ok(OrderType::PostOnly),
        FILLORKILL_ORDER_TYPE_STR => Ok(OrderType::FillOrKill),
        _ => Err(format!(
            "unknown order type string ({}), cannot map to OrderType",
            order_type_str
        )),
    }
}

const BID_ORDER_SIDE_STR: &str = "buy";
const ASK_ORDER_SIDE_STR: &str = "sell";

pub fn order_side_to_request_string(order_side: OrderSide) -> String {
    match order_side {
        OrderSide::BID => BID_ORDER_SIDE_STR.to_string(),
        OrderSide::ASK => ASK_ORDER_SIDE_STR.to_string(),
    }
}

fn string_to_order_side(order_side_str: &str) -> Result<OrderSide, String> {
    match order_side_str {
        BID_ORDER_SIDE_STR => Ok(OrderSide::BID),
        ASK_ORDER_SIDE_STR => Ok(OrderSide::ASK),
        _ => Err(format!(
            "unknown order side string ({}), cannot map to OrderSide",
            order_side_str
        )),
    }
}

const DONE_ORDER_STATE_STR: &str = "d";
const WAIT_ORDER_STATE_STR: &str = "w";
const CANCEL_ORDER_STATE_STR: &str = "c";
const REJECT_ORDER_STATE_STR: &str = "r";

pub fn order_state_to_request_string(order_state: OrderState) -> String {
    match order_state {
        OrderState::DONE => DONE_ORDER_STATE_STR.to_string(),
        OrderState::WAIT => WAIT_ORDER_STATE_STR.to_string(),
        OrderState::CANCEL => CANCEL_ORDER_STATE_STR.to_string(),
        OrderState::REJECT => REJECT_ORDER_STATE_STR.to_string(),
    }
}

fn string_to_order_state(order_state_str: &str) -> Result<OrderState, String> {
    match order_state_str {
        DONE_ORDER_STATE_STR => Ok(OrderState::DONE),
        WAIT_ORDER_STATE_STR => Ok(OrderState::WAIT),
        CANCEL_ORDER_STATE_STR => Ok(OrderState::CANCEL),
        REJECT_ORDER_STATE_STR => Ok(OrderState::REJECT),
        _ => Err(format!(
            "unknown order state string ({}), cannot map to OrderState",
            order_state_str
        )),
    }
}

pub mod asset_id_mapping {

    use super::*;

    const POLKADEX_ASSET_STR: &str = "pdx";
    const DOT_ASSET_STR: &str = "dot";
    const CHAIN_SAFE_ASSET_STR: &str = "chs";
    const BTC_ASSET_STR: &str = "btc";
    const USD_ASSET_STR: &str = "usd";

    pub fn asset_id_to_string(asset_id: AssetId) -> String {
        match asset_id {
            AssetId::POLKADEX => POLKADEX_ASSET_STR.to_string(),
            AssetId::Asset(0) => DOT_ASSET_STR.to_string(),

            // TODO: the string representation for these might have to include the hash?
            AssetId::Asset(1)(_) => CHAIN_SAFE_ASSET_STR.to_string(),
            AssetId::Asset(2) => BTC_ASSET_STR.to_string(),
            AssetId::Asset(3) => USD_ASSET_STR.to_string(),
        }
    }

    pub fn string_to_asset_id(asset_id_str: &str) -> Result<AssetId, String> {
        // TODO: we're using just dummy values here
        let dummy_token_hash = dummy_hash();

        match asset_id_str {
            POLKADEX_ASSET_STR => Ok(AssetId::POLKADEX),
            DOT_ASSET_STR => Ok(AssetId::Asset(0)),

            CHAIN_SAFE_ASSET_STR => Ok(AssetId::Asset(1)),
            BTC_ASSET_STR => Ok(AssetId::Asset(2)),
            USD_ASSET_STR => Ok(AssetId::Asset(3)),
            _ => Err(format!(
                "unknown asset id string ({}), cannot map to AssetId",
                asset_id_str
            )),
        }
    }

    pub fn dummy_hash() -> H160 {
        H160::from([2u8; 20])
    }
}

pub mod tests {

    use super::*;
    use crate::openfinex::market::{Market, MarketState};
    use sp_core::{ed25519 as ed25519_core, Pair};

    pub fn test_market_type_encoded_returns_correct_string() {
        let expected_market_type_str = "trusted".to_string();
        let market_type_encoded = expected_market_type_str.clone().into_bytes(); // use utf-8 encoding!
        let market_type_decoded = market_type_to_request_string(market_type_encoded).unwrap();

        assert_eq!(expected_market_type_str, market_type_decoded);
    }

    pub fn test_user_id_encoded_returns_correct_string() {
        let key_pair = ed25519_core::Pair::from_seed(b"12345678901234567890123456789012");
        let user_id: UserId = key_pair.public().into();

        let user_id_as_str = user_id_to_request_string(&user_id);

        assert!(!user_id_as_str.is_empty());
    }

    pub fn test_map_asset_ids() {
        let dummy_hash = H160::from([2u8; 20]);
        let asset_ids = vec![
            AssetId::Asset(0),
            AssetId::POLKADEX,
            AssetId::Asset(3),
            AssetId::Asset(2),
            AssetId::Asset(1),
        ];

        for asset_id in asset_ids {
            let asset_id_str = asset_id_mapping::asset_id_to_string(asset_id);
            let mapped_asset_id = asset_id_mapping::string_to_asset_id(&asset_id_str).unwrap();
            assert_eq!(asset_id, mapped_asset_id);
        }
    }

    pub fn test_map_order_side() {
        let order_sides = vec![OrderSide::ASK, OrderSide::BID];

        for order_side in order_sides {
            let order_side_str = order_side_to_request_string(order_side);
            let mapped_order_side = string_to_order_side(&order_side_str).unwrap();
            assert_eq!(order_side, mapped_order_side);
        }
    }

    pub fn test_map_order_type() {
        let order_types = vec![
            OrderType::MARKET,
            OrderType::LIMIT,
            OrderType::FillOrKill,
            OrderType::PostOnly,
        ];

        for order_type in order_types {
            let order_type_str = order_type_to_request_string(order_type);
            let mapped_order_type = string_to_order_type(&order_type_str).unwrap();
            assert_eq!(order_type, mapped_order_type);
        }
    }

    pub fn test_map_order_state() {
        let order_states = vec![
            OrderState::DONE,
            OrderState::REJECT,
            OrderState::CANCEL,
            OrderState::WAIT,
        ];

        for order_state in order_states {
            let order_state_str = order_state_to_request_string(order_state);
            let mapped_order_state = string_to_order_state(&order_state_str).unwrap();
            assert_eq!(order_state, mapped_order_state);
        }
    }

    pub fn test_map_market_id() {
        let mut market_cache = MarketCache::new();
        market_cache.set_markets(
            market_cache.request_id(),
            vec![
                create_market("dotdot", "dot", "dot"),
                create_market("pdxdot", "pdx", "dot"),
                create_market("chspdx", "chs", "pdx"),
                create_market("pdxbtc", "pdx", "btc"),
                create_market("btcusd", "btc", "usd"),
                create_market("usdpdx", "usd", "pdx"),
            ],
        );

        let market_ids = vec![
            MarketId {
                base: AssetId::Asset(0),
                quote: AssetId::Asset(0),
            },
            MarketId {
                base: AssetId::POLKADEX,
                quote: AssetId::Asset(0),
            },
            MarketId {
                base: AssetId::Asset(1),
                quote: AssetId::POLKADEX,
            },
            MarketId {
                base: AssetId::POLKADEX,
                quote: AssetId::Asset(2),
            },
            MarketId {
                base: AssetId::Asset(2),
                quote: AssetId::Asset(3),
            },
            MarketId {
                base: AssetId::Asset(3),
                quote: AssetId::POLKADEX,
            },
        ];

        for market_id in market_ids {
            let market_id_str = market_id_to_request_string(market_id);
            let mapped_market_id = string_to_market_id(&market_id_str, &market_cache).unwrap();
            assert_eq!(market_id, mapped_market_id);
        }
    }

    fn create_market(id: &str, base: &str, quote: &str) -> Market {
        Market {
            id: String::from(id),
            base_unit: String::from(base),
            quote_unit: String::from(quote),
            state: MarketState::enabled,
            price_precision: 4,
            amount_precision: 4,
            name: String::from(id),
        }
    }
}
