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

use crate::cli_utils::account_parsing::get_identifiers;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::commands::common_args::{add_main_account_args, add_order_args, add_proxy_account_args};
use crate::{KeyPair, TrustedCall, TrustedOperation};
use clap::{App, ArgMatches};
use clap_nested::Command;

use crate::cli_utils::common_types::OperationRunner;
use crate::commands::account_details::AccountDetails;
use crate::commands::common_args_processing::get_order_from_matches;

pub fn cancel_order_cli_command(perform_operation: OperationRunner) -> Command<str> {
    Command::new("cancel_order")
        .description("Cancel order")
        .options(|app| add_app_args(app))
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            command_runner(matches, perform_operation)
        })
}

fn add_app_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let app_with_main_account = add_main_account_args(app);
    let app_with_proxy_account = add_proxy_account_args(app_with_main_account);
    add_order_args(app_with_proxy_account)
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

    let order =
        get_order_from_matches(matches).expect("failed to build order from command line arguments");

    let direct: bool = matches.is_present("direct");

    let cancel_order_top: TrustedOperation = TrustedCall::cancel_order(
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

    let _ = perform_operation(matches, &cancel_order_top);

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{Getter, Index, TrustedCallSigned, TrustedGetter, TrustedGetterSigned};
    use clap::{App, AppSettings};
    use clap_nested::{CommandLike, Commander};
    use codec::Encode;

    #[test]
    fn given_the_proper_arguments_then_run_operation() {
        let proxy_account_arg = format!("--{}=oijfw93ojafje3k", PROXY_ACCOUNT_ID_ARG_NAME);
        let arg_vec = vec![proxy_account_arg];

        // trusted set-balance //AliceIncognito 123456789 --mrenclave $MRENCLAVE --direct

        let matches = App::new("test_program")
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(arg_vec);

        let command = cancel_order_cli_command(&perform_operation_mock);

        let commander = Commander::new()
            .args(|_args, matches| matches.value_of("environment").unwrap_or("dev"))
            .add_cmd(command)
            .no_cmd(|_args, _matches| {
                println!("No subcommand matched");
                Ok(())
            });

        let result = commander.run();

        assert!(result.is_ok());
    }

    fn perform_operation_mock(
        arg_matches: &ArgMatches<'_>,
        trusted_operation: &TrustedOperation,
    ) -> Option<Vec<u8>> {
        match trusted_operation {
            TrustedOperation::indirect_call(tcs) => {}
            TrustedOperation::direct_call(tcs) => {}
            TrustedOperation::get(get) => match get {
                Getter::public(_) => {}
                Getter::trusted(tgs) => match &tgs.getter {
                    TrustedGetter::nonce(accountId) => {
                        return Some(Index::encode(&145));
                    }
                    TrustedGetter::free_balance(_) => {}
                    TrustedGetter::reserved_balance(_) => {}
                },
            },
        }
        None
    }
}
