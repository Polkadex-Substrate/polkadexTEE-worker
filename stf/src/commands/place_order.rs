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

use crate::cli_utils::account_parsing::*;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::{KeyPair, TrustedCall, TrustedOperation};

use crate::commands::account_details::AccountDetails;
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
            let account_details = AccountDetails::new(matches);

            let signer_pair = account_details.signer_pair();
            let signer_key_pair = account_details.signer_key_pair();

            let (mrenclave, shard) = get_identifiers(matches);
            let nonce =
                get_trusted_nonce(perform_operation, matches, &signer_pair, &signer_key_pair);

            let order = get_order_from_matches(matches)
                .expect("failed to build order from command line arguments");

            let direct: bool = matches.is_present("direct");

            let place_order_top: TrustedOperation = TrustedCall::place_order(
                account_details.main_account_public_key().into(),
                order,
                account_details
                    .proxy_account_public_key()
                    .map(|pk| pk.into()),
            )
            .sign(
                &KeyPair::Sr25519(signer_key_pair),
                nonce,
                &mrenclave,
                &shard,
            )
            .into_trusted_operation(direct);

            let _ = perform_operation(matches, &place_order_top);

            Ok(())
        })
}
