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

use clap::{App, ArgMatches};
use clap_nested::Command;
use codec::Encode;
use log::*;

use crate::cli_utils::account_parsing::*;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::{KeyPair, TrustedCall, TrustedOperation};

use crate::cli_utils::common_types::OperationRunner;
use crate::commands::account_details::AccountDetails;
use crate::commands::common_args::*;
use crate::commands::common_args_processing::get_token_id_from_matches;

pub fn get_balance_cli_command<'a>(
    perform_operation: &'a dyn Fn(&ArgMatches<'_>, &TrustedOperation) -> Option<Vec<u8>>,
) -> Command<'a, str> {
    Command::new("get_balance")
        .description("Get the balance")
        .options(|app| add_command_args(app))
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            command_runner(matches, perform_operation)
        })
}

pub fn add_command_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let app_with_main_account = add_main_account_args(app);
    let app_with_proxy_account = add_proxy_account_args(app_with_main_account);
    add_token_id_args(app_with_proxy_account)
}

fn command_runner<'a>(
    matches: &ArgMatches<'_>,
    perform_operation: OperationRunner<'a>,
) -> Result<(), clap::Error> {
    let account_details = AccountDetails::new(matches);

    let signer_pair = account_details.signer_pair();
    let signer_key_pair = account_details.signer_key_pair();

    let (mrenclave, shard) = get_identifiers(matches);
    let nonce = get_trusted_nonce(perform_operation, matches, &signer_pair, &signer_key_pair);

    let direct: bool = matches.is_present("direct");

    let currency_id = get_token_id_from_matches(matches).unwrap();

    let get_balance_top: TrustedOperation = TrustedCall::get_balance(
        account_details.signer_public_key().into(),
        currency_id.encode(),
        account_details
            .main_account_public_key_if_not_signer()
            .map(|pk| pk.into()),
    )
    .sign(
        &KeyPair::Sr25519(signer_key_pair),
        nonce,
        &mrenclave,
        &shard,
    )
    .into_trusted_operation(direct);

    debug!("Successfully built get_balance trusted operation, dispatching now to enclave");

    let _ = perform_operation(matches, &get_balance_top);

    debug!("get_balance trusted operation was executed");

    Ok(())
}
