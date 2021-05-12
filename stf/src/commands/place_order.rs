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

use clap::{Arg, ArgMatches};
use clap_nested::Command;
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair};

use crate::cli_utils::account_parsing::*;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::order::Order;
use crate::{KeyPair, TrustedCall, TrustedGetter, TrustedOperation};

pub fn place_order_cli_command<'a>(
    perform_operation: &'a dyn Fn(&ArgMatches<'_>, &TrustedOperation) -> Option<Vec<u8>>,
) -> Command<'a, str> {
    const ACCOUNT_ID_ARG_NAME: &str = "accountid";
    const PROXY_ACCOUNT_ID_ARG_NAME: &str = "proxyaccountid";
    const MIN_ORDER_AMOUNT_ARG_NAME: &str = "min";
    const MAX_ORDER_AMOUNT_ARG_NAME: &str = "max";
    const CURRENCY_ID_ARG_NAME: &str = "currencyid";

    Command::new("place-order")
        .description("Place order to ... TODO")
        .options(|app| {
            app.arg(
                Arg::with_name(ACCOUNT_ID_ARG_NAME)
                    .takes_value(true)
                    .required(true)
                    .value_name("SS58")
                    .help("AccountId in ss58check format"),
            )
            .arg(
                Arg::with_name(PROXY_ACCOUNT_ID_ARG_NAME)
                    .takes_value(true)
                    .required(true)
                    .value_name("SS58")
                    .help("Proxy AccountId in ss58check format"),
            )
            .arg(
                Arg::with_name(MIN_ORDER_AMOUNT_ARG_NAME)
                    .takes_value(true)
                    .required(true)
                    .value_name("u128")
                    .help("Min amount to trade"),
            )
            .arg(
                Arg::with_name(MAX_ORDER_AMOUNT_ARG_NAME)
                    .takes_value(true)
                    .required(true)
                    .value_name("u128")
                    .help("Max amount to trade"),
            )
            .arg(
                Arg::with_name(CURRENCY_ID_ARG_NAME)
                    .takes_value(true)
                    .required(true)
                    .value_name("u8")
                    .help("Currency"),
            )
        })
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            let arg_account = matches.value_of(ACCOUNT_ID_ARG_NAME).unwrap();

            let main_account_pair = get_pair_from_str(matches, arg_account);
            let main_account_key_pair = sr25519_core::Pair::from(main_account_pair.clone());
            let main_account_public_key = sr25519_core::Public::from(main_account_pair.public());

            let proxy_account_pair = get_pair_from_str(
                matches,
                matches.value_of(PROXY_ACCOUNT_ID_ARG_NAME).unwrap(),
            );
            //let proxy_account_key_pair = sr25519_core::Pair::from(proxy_account_pair.clone());
            let proxy_account_public_key = sr25519_core::Public::from(proxy_account_pair.public());

            let min_amount = get_amount_from_matches(matches, MIN_ORDER_AMOUNT_ARG_NAME);
            let max_amount = get_amount_from_matches(matches, MAX_ORDER_AMOUNT_ARG_NAME);
            let currency_id = get_currency_id_from_matches(matches, CURRENCY_ID_ARG_NAME);

            let (mrenclave, shard) = get_identifiers(matches);
            let nonce = get_trusted_nonce(
                perform_operation,
                matches,
                &main_account_pair,
                &main_account_key_pair,
            );

            let order = Order {
                min_amount,
                max_amount,
                currency_id,
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

// would like to use num_traits::Num here to parameterize the function, but it doesn't seem to be available

fn get_currency_id_from_matches(matches: &ArgMatches<'_>, arg_name: &str) -> u8 {
    u8::from_str_radix(matches.value_of(arg_name).unwrap(), 10)
        .expect(&format!("failed to convert {} into an integer", arg_name))
}

fn get_amount_from_matches(matches: &ArgMatches<'_>, arg_name: &str) -> u128 {
    u128::from_str_radix(matches.value_of(arg_name).unwrap(), 10)
        .expect(&format!("failed to convert {} into an integer", arg_name))
}

//
// fn get_integer_value_from_matches<T>(matches: &ArgMatches<'_>, arg_name: &str) -> T
// where
//     T: FromStrRadix
// {
//     T::from_str_radix(matches.value_of(arg_name).unwrap(), 10)
//         .expect(&format!("failed to convert {} into an integer", arg_name))
// }
