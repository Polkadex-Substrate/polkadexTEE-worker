/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/

use clap::{App, Arg};

pub const ACCOUNT_ID_ARG_NAME: &str = "accountid";
pub const PROXY_ACCOUNT_ID_ARG_NAME: &str = "proxyaccountid";
pub const MARKET_ID_ARG_NAME: &str = "marketid";
pub const MARKET_TYPE_ARG_NAME: &str = "markettype";
pub const ORDER_TYPE_ARG_NAME: &str = "ordertype";
pub const ORDER_SIDE_ARG_NAME: &str = "orderside";
pub const QUANTITY_ARG_NAME: &str = "quantity";
pub const PRICE_ARG_NAME: &str = "price";

pub fn add_main_account_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(ACCOUNT_ID_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Account/User ID"),
    )
}

pub fn add_proxy_account_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(PROXY_ACCOUNT_ID_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Proxy Account ID"),
    )
}

pub fn add_order_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(MARKET_ID_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Market ID, e.g.: 'btcusd'"),
    )
    .arg(
        Arg::with_name(MARKET_TYPE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Market type, e.g. 'trusted'"),
    )
    .arg(
        Arg::with_name(ORDER_TYPE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Order type: one of [market, limit, postonly, fillorkill]"),
    )
    .arg(
        Arg::with_name(QUANTITY_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("u128")
            .help("An amount that placed within the order"),
    )
    .arg(
        Arg::with_name(PRICE_ARG_NAME)
            .takes_value(true)
            .required(false)
            .value_name("u128")
            .help("Main (limit) price of the order (optional)"),
    )
}
