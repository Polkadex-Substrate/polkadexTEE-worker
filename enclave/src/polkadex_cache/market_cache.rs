// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü.
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

use crate::openfinex::market::Market;
use crate::openfinex::openfinex_types::RequestId;
use crate::polkadex_cache::cache_api::LocalCacheProvider;
use log::*;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

pub struct LocalMarketCacheFactory {}

/// factory for a local cache of MarketCache
impl LocalMarketCacheFactory {
    pub fn create() -> LocalCacheProvider<MarketCache> {
        LocalCacheProvider::<MarketCache>::new(&|| MarketCache::new())
    }
}

#[derive(Debug, Clone)]
pub struct MarketCache {
    /// The set of cached markets
    markets: HashMap<String, Market>,

    /// current request ID - increments after receiving a markets update
    request_id: RequestId,
}

impl Default for MarketCache {
    fn default() -> Self {
        MarketCache {
            markets: Default::default(),
            request_id: 0,
        }
    }
}

impl MarketCache {
    /// constructor
    pub fn new() -> MarketCache {
        MarketCache {
            markets: HashMap::new(),
            request_id: 1,
        }
    }

    /// set markets in the cache, replacing any existing ones
    /// checks if the request ids match, otherwise do nothing
    pub fn set_markets(&mut self, request_id: RequestId, markets: Vec<Market>) {
        if self.request_id != request_id {
            warn!("Attempting to update markets, but request ID does not match; ignoring attempt");
            return;
        }

        self.markets.clear();
        for market in markets {
            self.markets.insert(market.id.clone(), market);
        }

        self.increment_request_id();
    }

    /// gets a market by id
    pub fn get_market(&self, id: &String) -> Option<&Market> {
        self.markets.get(id)
    }

    pub fn request_id(&self) -> RequestId {
        self.request_id
    }
}

impl MarketCache {
    fn increment_request_id(&mut self) {
        self.request_id = self.request_id.wrapping_add(1)
    }
}

pub mod tests {

    use super::*;
    use crate::openfinex::market::MarketState;
    use std::string::ToString;

    pub fn set_markets_with_valid_request_id() {
        let markets = vec![
            create_market("btcusd", "btc", "usd"),
            create_market("usdbtc", "usd", "btc"),
        ];

        let mut market_cache = MarketCache {
            markets: HashMap::new(),
            request_id: 1,
        };
        market_cache.set_markets(market_cache.request_id(), markets.clone());

        assert_eq!(2, market_cache.request_id());
        assert_eq!(markets.len(), market_cache.markets.len());
    }

    pub fn set_markets_with_invalid_request_id() {
        let markets = vec![create_market("example", "la", "di")];

        let mut market_cache = MarketCache {
            markets: HashMap::new(),
            request_id: 1,
        };
        market_cache.set_markets(0, markets); // invalid/non-matching request id

        assert_eq!(1, market_cache.request_id());
        assert!(market_cache.markets.is_empty()); // still empty, because request ID didn-t match
    }

    pub fn set_markets_repeatedly_clears_previous() {
        let markets = vec![create_market("market_id", "base", "quote")];

        let mut market_cache = MarketCache {
            markets: HashMap::new(),
            request_id: 569,
        };

        market_cache.set_markets(market_cache.request_id(), markets);
        market_cache.set_markets(market_cache.request_id(), vec![]);

        assert_eq!(571, market_cache.request_id);
        assert!(market_cache.markets.is_empty());
    }

    pub fn retrieve_previously_inserted_markets() {
        let markets = vec![
            create_market("btcusd", "btc", "usd"),
            create_market("usdbtc", "usd", "btc"),
            create_market("trsteth", "trst", "eth"),
        ];

        let mut market_cache = MarketCache {
            markets: HashMap::new(),
            request_id: 2345,
        };

        market_cache.set_markets(market_cache.request_id(), markets);

        assert!(market_cache.get_market(&"btcusd".to_string()).is_some());
        assert!(market_cache.get_market(&"trsteth".to_string()).is_some());
        assert!(market_cache
            .get_market(&"other_invalid".to_string())
            .is_none());
    }

    fn create_market(id: &str, base_unit: &str, quote_unit: &str) -> Market {
        Market {
            id: String::from(id),
            base_unit: String::from(base_unit),
            quote_unit: String::from(quote_unit),
            state: MarketState::enabled,
            name: String::from(id),
            amount_precision: 4,
            price_precision: 4,
        }
    }
}
