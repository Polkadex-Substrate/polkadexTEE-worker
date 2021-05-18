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

use core::option::Option;

use clap::{Arg, ArgMatches};
use clap_nested::Command;
use codec::Encode;

use sp_core::{sr25519 as sr25519_core, Pair};

use crate::cli_utils::account_parsing::*;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::{KeyPair, TrustedCall, TrustedOperation};
use std::error::Error;

use polkadex_sgx_primitives::types::{Order, OrderSide, OrderType};

pub fn place_order_cli_command<'a>(
    perform_operation: &'a dyn Fn(&ArgMatches<'_>, &TrustedOperation) -> Option<Vec<u8>>,
) -> Command<'a, str> {
    const ACCOUNT_ID_ARG_NAME: &str = "accountid";
    const PROXY_ACCOUNT_ID_ARG_NAME: &str = "proxyaccountid";
    const MARKET_ID_ARG_NAME: &str = "marketid";
    const MARKET_TYPE_ARG_NAME: &str = "markettype";
    const ORDER_TYPE_ARG_NAME: &str = "ordertype";
    const ORDER_SIDE_ARG_NAME: &str = "orderside";
    const QUANTITY_ARG_NAME: &str = "quantity";
    const PRICE_ARG_NAME: &str = "price";

    Command::new("place_order")
        .description("Place order")
        .options(|app| {
            app.arg(
                Arg::with_name(ACCOUNT_ID_ARG_NAME)
                    .takes_value(true)
                    .required(true)
                    .value_name("STRING")
                    .help("Account/User ID"),
            )
            .arg(
                Arg::with_name(PROXY_ACCOUNT_ID_ARG_NAME)
                    .takes_value(true)
                    .required(true)
                    .value_name("STRING")
                    .help("Proxy Account ID"),
            )
            .arg(
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
        })
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            let arg_account = matches.value_of(ACCOUNT_ID_ARG_NAME).unwrap();
            let arg_market_id = matches.value_of(MARKET_ID_ARG_NAME).unwrap();
            let arg_market_type = matches.value_of(MARKET_TYPE_ARG_NAME).unwrap();

            let main_account_pair = get_pair_from_str(matches, arg_account);
            let main_account_key_pair = sr25519_core::Pair::from(main_account_pair.clone());
            let main_account_public_key = sr25519_core::Public::from(main_account_pair.public());

            let proxy_account_pair = get_pair_from_str(
                matches,
                matches.value_of(PROXY_ACCOUNT_ID_ARG_NAME).unwrap(),
            );
            //let proxy_account_key_pair = sr25519_core::Pair::from(proxy_account_pair.clone());
            let proxy_account_public_key = sr25519_core::Public::from(proxy_account_pair.public());

            let (mrenclave, shard) = get_identifiers(matches);
            let nonce = get_trusted_nonce(
                perform_operation,
                matches,
                &main_account_pair,
                &main_account_key_pair,
            );

            let arg_order_type =
                get_order_type_from_str(matches.value_of(ORDER_TYPE_ARG_NAME).unwrap()).expect("");

            let arg_order_side =
                get_order_side_from_str(matches.value_of(ORDER_SIDE_ARG_NAME).unwrap()).expect("");

            let arg_quantity = get_amount_from_matches(matches, QUANTITY_ARG_NAME);
            let arg_price = matches
                .value_of(PRICE_ARG_NAME)
                .map(|v| get_amount_from_str(v));

            let order = Order {
                user_uid: arg_account.encode(),
                market_id: arg_market_id.encode(),
                market_type: arg_market_type.encode(),
                order_type: arg_order_type,
                side: arg_order_side,
                quantity: arg_quantity,
                price: arg_price,
            };

            let direct: bool = matches.is_present("direct");

            let place_order_top: TrustedOperation = TrustedCall::place_order(
                main_account_public_key.into(),
                order,
                proxy_account_public_key.into(),
            )
            .sign(
                &KeyPair::Sr25519(main_account_key_pair),
                nonce,
                &mrenclave,
                &shard,
            )
            .into_trusted_operation(direct);

            let _ = perform_operation(matches, &place_order_top);

            Ok(())
        })
}

fn get_amount_from_matches(matches: &ArgMatches<'_>, arg_name: &str) -> u128 {
    get_amount_from_str(matches.value_of(arg_name).unwrap())
}

fn get_amount_from_str(arg: &str) -> u128 {
    u128::from_str_radix(arg, 10).expect(&format!("failed to convert {} into an integer", arg))
}

fn get_order_type_from_str(arg: &str) -> Result<OrderType, &str> {
    match arg.to_ascii_lowercase().as_ref() {
        "limit" => Ok(OrderType::LIMIT),
        "market" => Ok(OrderType::MARKET),
        "postonly" => Ok(OrderType::PostOnly),
        "fillorkill" => Ok(OrderType::FillOrKill),
        _ => Err("invalid order type argument"),
    }
}

fn get_order_side_from_str(arg: &str) -> Result<OrderSide, &str> {
    match arg.to_ascii_lowercase().as_ref() {
        "bid" => Ok(OrderSide::BID),
        "ask" => Ok(OrderSide::ASK),
        _ => Err("invalid order side argument"),
    }
}
