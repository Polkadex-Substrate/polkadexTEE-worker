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
        account_details.signer_public_key().into(),
        order,
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

    let _ = perform_operation(matches, &cancel_order_top);

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::commands::test_utils::utils::{
        add_identifiers_app_args, create_identifier_args, create_main_account_args,
        create_order_args, PerformOperationMock,
    };
    use clap::{App, AppSettings};

    #[test]
    fn given_the_proper_arguments_then_run_operation() {
        let args = create_cancel_order_args();
        let matches = create_test_app().get_matches_from(args);

        let perform_operation_mock = PerformOperationMock::new();

        let command_result = command_runner(
            &matches,
            &|arg_matches: &ArgMatches<'_>, top: &TrustedOperation| {
                perform_operation_mock.perform_operation_mock(arg_matches, top)
            },
        );

        assert!(command_result.is_ok());
    }

    fn create_cancel_order_args() -> Vec<String> {
        let mut main_account_arg = create_main_account_args();
        let mut order_args = create_order_args();
        let mut identifier_args = create_identifier_args();

        main_account_arg.append(&mut order_args);
        main_account_arg.append(&mut identifier_args);

        main_account_arg
    }

    fn create_test_app<'a, 'b>() -> App<'a, 'b> {
        let test_app = App::new("test_account_details").setting(AppSettings::NoBinaryName);
        let app_with_arg = add_app_args(test_app);

        add_identifiers_app_args(app_with_arg)
    }
}
