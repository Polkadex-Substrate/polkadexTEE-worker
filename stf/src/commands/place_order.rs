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

use clap::ArgMatches;
use clap_nested::Command;

use sp_core::{sr25519 as sr25519_core, Pair};

use crate::cli_utils::account_parsing::*;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::{KeyPair, TrustedCall, TrustedOperation};
use std::error::Error;

use crate::commands::common_args::*;
use crate::commands::common_args_processing::get_order_from_matches;

pub fn place_order_cli_command<'a>(
    perform_operation: &'a dyn Fn(&ArgMatches<'_>, &TrustedOperation) -> Option<Vec<u8>>,
) -> Command<'a, str> {
    Command::new("place_order")
        .description("Place order")
        .options(|app| {
            let app_with_main_account = add_main_account_args(app);
            let app_with_proxy_account = add_proxy_account_args(app_with_main_account);
            add_order_args(app_with_proxy_account)
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

            let (mrenclave, shard) = get_identifiers(matches);
            let nonce = get_trusted_nonce(
                perform_operation,
                matches,
                &main_account_pair,
                &main_account_key_pair,
            );

            let order = get_order_from_matches(arg_account, matches)
                .expect("failed to build order from command line arguments");

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
