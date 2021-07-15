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

use clap::{App, Arg};

pub const ACCOUNT_ID_ARG_NAME: &str = "accountid";
pub const PROXY_ACCOUNT_ID_ARG_NAME: &str = "proxyaccountid";
pub const MARKET_ID_BASE_ARG_NAME: &str = "marketbase";
pub const MARKET_ID_QUOTE_ARG_NAME: &str = "marketquote";
pub const MARKET_TYPE_ARG_NAME: &str = "markettype";
pub const ORDER_TYPE_ARG_NAME: &str = "ordertype";
pub const ORDER_SIDE_ARG_NAME: &str = "orderside";
pub const QUANTITY_ARG_NAME: &str = "quantity";
pub const PRICE_ARG_NAME: &str = "price";
pub const TOKEN_ID_ARG_NAME: &str = "tokenid";
pub const ORDER_UUID_ARG_NAME: &str = "orderid";
pub const MRENCLAVE_ARG_NAME: &str = "mrenclave";
pub const SHARD_ARG_NAME: &str = "shard";

pub fn add_common_order_command_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let app_with_main_account = add_main_account_args(app);
    let app_with_proxy_account = add_proxy_account_args(app_with_main_account);
    add_order_args(app_with_proxy_account)
}

pub fn add_token_id_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(TOKEN_ID_ARG_NAME)
            .long(TOKEN_ID_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Token (i.e. currency) ID, e.g. "),
    )
}

pub fn add_quantity_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(QUANTITY_ARG_NAME)
            .long(QUANTITY_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("u128")
            .help("specifies the amount of funds/tokens"),
    )
}

pub fn add_main_account_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(ACCOUNT_ID_ARG_NAME)
            .long(ACCOUNT_ID_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("SS58")
            .help("Account/User ID"),
    )
}

pub fn add_proxy_account_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(PROXY_ACCOUNT_ID_ARG_NAME)
            .long(PROXY_ACCOUNT_ID_ARG_NAME)
            .takes_value(true)
            .required(false) // proxy account is optional
            .value_name("SS58")
            .help("Proxy Account ID"),
    )
}

pub fn add_order_id_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(ORDER_UUID_ARG_NAME)
            .long(ORDER_UUID_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("UUID STRING")
            .help("Order UUID"),
    )
}

pub fn add_market_id_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(MARKET_ID_BASE_ARG_NAME)
            .long(MARKET_ID_BASE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Market base asset ID, e.g.: 'polkadex', 'dot'"),
    )
    .arg(
        Arg::with_name(MARKET_ID_QUOTE_ARG_NAME)
            .long(MARKET_ID_QUOTE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Market quote asset ID, e.g.: 'polkadex', 'dot'"),
    )
}

pub fn add_order_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(MARKET_ID_BASE_ARG_NAME)
            .long(MARKET_ID_BASE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Market base asset ID, e.g.: 'polkadex', 'dot'"),
    )
    .arg(
        Arg::with_name(MARKET_ID_QUOTE_ARG_NAME)
            .long(MARKET_ID_QUOTE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Market quote asset ID, e.g.: 'polkadex', 'dot'"),
    )
    .arg(
        Arg::with_name(MARKET_TYPE_ARG_NAME)
            .long(MARKET_TYPE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Market type, e.g. 'trusted'"),
    )
    .arg(
        Arg::with_name(ORDER_TYPE_ARG_NAME)
            .long(ORDER_TYPE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Order type: one of [market, limit, postonly, fillorkill]"),
    )
    .arg(
        Arg::with_name(ORDER_SIDE_ARG_NAME)
            .long(ORDER_SIDE_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("Order side: one of [bid, ask]"),
    )
    .arg(
        Arg::with_name(QUANTITY_ARG_NAME)
            .long(QUANTITY_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("u128")
            .help("An amount that placed within the order"),
    )
    .arg(
        Arg::with_name(PRICE_ARG_NAME)
            .long(PRICE_ARG_NAME)
            .takes_value(true)
            .required(false)
            .value_name("u128")
            .help("Main (limit) price of the order (optional)"),
    )
}
