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

use core::option::Option;

use clap::ArgMatches;
use clap_nested::Command;
use log::*;

use crate::cli_utils::account_parsing::*;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::{KeyPair, TrustedCall, TrustedOperation};

use crate::cli_utils::common_types::OperationRunner;
use crate::commands::account_details::AccountDetails;
use crate::commands::common_args::*;
use crate::commands::common_args_processing::get_order_from_matches;

pub fn place_order_cli_command<'a>(
    perform_operation: &'a dyn Fn(&ArgMatches<'_>, &TrustedOperation) -> Option<Vec<u8>>,
) -> Command<'a, str> {
    Command::new("place_order")
        .description("Place order")
        .options(add_common_order_command_args)
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            command_runner(matches, perform_operation)
        })
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

    let order = get_order_from_matches(matches, account_details.main_account_public_key().into())
        .expect("failed to build order from command line arguments");

    let direct: bool = matches.is_present("direct");

    let place_order_top: TrustedOperation = TrustedCall::place_order(
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

    debug!("Successfully built place_order trusted operation, dispatching now to enclave");

    //let _ = perform_operation(matches, &place_order_top);

    // let bal = if let Some(v) = perform_operation(matches, &place_order_top) {
    //         v
    //     } ;
    let bal = perform_operation(matches, &place_order_top).unwrap();

    println!("{:?}", bal);
    debug!("place_order trusted operation was executed");

    Ok(())
}
