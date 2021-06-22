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

use serde::{Deserialize, Serialize};
use std::{string::String, string::ToString};

//{"id":"btcusd","name":"BTC/USD","base_unit":"btc","quote_unit":"usd","state":"enabled","amount_precision":4,
// "price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":100,"filters":[]}

/// Market object given by OpenFinex as JSON string
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Market {
    pub id: String,
    pub name: String,
    pub base_unit: String,
    pub quote_unit: String,
    pub state: MarketState,
    pub amount_precision: u128,
    pub price_precision: u128,
    //min_price: String,
    //max_price: String,
    //min_amount: String,
    //position: u128,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum MarketState {
    enabled,
    disabled,
}

pub mod tests {

    use super::*;

    pub fn test_deserialize_market_usdbtc() {
        let json_string = r#"{"id":"btcusd","name":"BTC/USD","base_unit":"btc","quote_unit":"usd","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":100,"filters":[]}"#.to_string();

        let market: Market = serde_json::from_str(json_string.as_str()).unwrap();
        assert_eq!("btcusd", market.id);
        assert_eq!("BTC/USD", market.name);
        assert_eq!("btc", market.base_unit);
        assert_eq!("usd", market.quote_unit);
        assert_eq!(MarketState::enabled, market.state);
    }

    pub fn test_deserialize_market_trsteth() {
        let json_string = r#"{"id":"trsteth","name":"TRST/ETH","base_unit":"trst","quote_unit":"eth","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":105,"filters":[]}"#.to_string();

        let market: Market = serde_json::from_str(json_string.as_str()).unwrap();
        assert_eq!("trsteth", market.id);
        assert_eq!("TRST/ETH", market.name);
        assert_eq!("trst", market.base_unit);
        assert_eq!("eth", market.quote_unit);
        assert_eq!(MarketState::enabled, market.state);
    }
}
