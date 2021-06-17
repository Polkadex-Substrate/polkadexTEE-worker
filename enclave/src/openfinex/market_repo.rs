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
use crate::openfinex::openfinex_types::{RequestId, RequestType};
use crate::openfinex::request_builder::OpenFinexRequestBuilder;
use crate::polkadex_cache::cache_api::CacheProvider;
use crate::polkadex_cache::market_cache::MarketCache;
use log::*;
use std::sync::Arc;
use std::{fmt::Display, fmt::Formatter, fmt::Result as FormatResult, string::String, vec::Vec};

#[derive(Eq, Debug, PartialOrd, PartialEq)]
pub enum MarketRepositoryError {
    FailedToLoadCache,
    FailedToLockCache,
}

impl Display for MarketRepositoryError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        write!(f, "{:?}", self)
    }
}

/// callback trait for receiving markets updates
pub trait MarketsRequestCallback {
    fn update_markets(
        &self,
        request_id: RequestId,
        json_strings: &Vec<String>,
    ) -> Result<(), MarketRepositoryError>;
}

/// trait for sending market update requests
pub trait MarketsRequestSender {
    fn get_markets_ws_request(&self) -> Result<String, MarketRepositoryError>;
}

/// a market repository, implementing requesting, receiving and storing markets updates from OpenFinex
pub struct MarketRepository {
    cache_provider: Arc<dyn CacheProvider<MarketCache>>,
}

impl MarketRepository {
    pub fn new(cache_provider: Arc<dyn CacheProvider<MarketCache>>) -> Self {
        MarketRepository { cache_provider }
    }
}

impl MarketsRequestCallback for MarketRepository {
    fn update_markets(
        &self,
        request_id: u128,
        json_strings: &Vec<String>,
    ) -> Result<(), MarketRepositoryError> {
        let mutex = self
            .cache_provider
            .load()
            .map_err(|_| MarketRepositoryError::FailedToLoadCache)?;
        let mut cache = mutex.lock().map_err(|e| {
            error!("Could not acquire lock on market cache pointer: {}", e);
            MarketRepositoryError::FailedToLockCache
        })?;

        let markets = json_strings
            .iter()
            .filter_map(|s| map_string_to_market(s))
            .collect();

        cache.set_markets(request_id, markets);

        Ok(())
    }
}

impl MarketsRequestSender for MarketRepository {
    fn get_markets_ws_request(&self) -> Result<String, MarketRepositoryError> {
        let mutex = self
            .cache_provider
            .load()
            .map_err(|_| MarketRepositoryError::FailedToLoadCache)?;
        let cache = mutex.lock().map_err(|e| {
            error!("Could not acquire lock on market cache pointer: {}", e);
            MarketRepositoryError::FailedToLockCache
        })?;
        let request_id = cache.request_id();

        let request_builder = OpenFinexRequestBuilder::new(RequestType::GetMarkets, request_id);

        let request = request_builder.build();
        Ok(request.to_request_string())
    }
}

fn map_string_to_market(json_string: &String) -> Option<Market> {
    let market: Option<Market> = match serde_json::from_str(json_string.as_str()) {
        Ok(m) => Some(m),
        Err(e) => {
            error!("Failed to deserialize string to a Market object: {}", e);
            None
        }
    };
    market
}
