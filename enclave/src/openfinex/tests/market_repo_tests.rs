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

use crate::openfinex::market_repo::{MarketRepository, MarketsRequestCallback};
use crate::polkadex_cache::cache_api::{CacheProvider, CacheResult};
use crate::polkadex_cache::market_cache::MarketCache;
use log::*;
use std::string::ToString;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, SgxMutex};

struct MarketCacheProviderMock {
    initial_market_cache: MarketCache,
    cache_ptr: AtomicPtr<()>,
}

impl MarketCacheProviderMock {}

impl CacheProvider<MarketCache> for MarketCacheProviderMock {
    fn initialize(&self) {
        let cache_storage_ptr = Arc::new(SgxMutex::new(self.initial_market_cache.clone()));
        let cache_ptr = Arc::into_raw(cache_storage_ptr);
        self.cache_ptr.store(cache_ptr as *mut (), Ordering::SeqCst);
    }

    fn load(&self) -> CacheResult<&'static SgxMutex<MarketCache>> {
        let ptr = self.cache_ptr.load(Ordering::SeqCst) as *mut SgxMutex<MarketCache>;
        if ptr.is_null() {
            error!("Could not load cache");
            return Err(());
        } else {
            Ok(unsafe { &*ptr })
        }
    }
}

pub fn update_markets_from_json_strings() {
    let cache_provider = Arc::new(MarketCacheProviderMock {
        initial_market_cache: MarketCache::new(),
        cache_ptr: AtomicPtr::new(0 as *mut ()),
    });
    cache_provider.initialize();

    let market_repo = MarketRepository::new(cache_provider);

    let json_strings = vec![
        r#"{"id":"btcusd","name":"BTC/USD","base_unit":"btc","quote_unit":"usd","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":100,"filters":[]}"#.to_string(),
        r#"{"id":"trsteth","name":"TRST/ETH","base_unit":"trst","quote_unit":"eth","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":105,"filters":[]}"#.to_string()
    ];

    assert!(market_repo.update_markets(1, &json_strings).is_ok());
}
