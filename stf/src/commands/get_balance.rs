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

use crate::{
    cli_utils::common_types::OperationRunner,
    commands::{
        account_details::AccountDetails, common_args::*,
        common_args_processing::get_token_id_from_matches,
    },
    KeyPair, TrustedGetter, TrustedOperation,
};
use clap::{App, ArgMatches};
use clap_nested::Command;
use codec::{Decode, Encode};
use core::option::Option;
use log::*;
use polkadex_sgx_primitives::Balance;

pub fn get_balance_cli_command<'a>(
    perform_operation: &'a dyn Fn(&ArgMatches<'_>, &TrustedOperation) -> Option<Vec<u8>>,
) -> Command<'a, str> {
    Command::new("get_balance")
        .description("Get the balance")
        .options(add_command_args)
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

    let signer_key_pair = account_details.signer_key_pair();

    let token_id = get_token_id_from_matches(matches).unwrap();

    let get_balance_top: TrustedOperation = TrustedGetter::get_balance(
        account_details.signer_public_key().into(),
        token_id,
        account_details
            .main_account_public_key_if_not_signer()
            .map(|pk| pk.into()),
    )
    .sign(&KeyPair::Sr25519(signer_key_pair))
    .into();

    debug!("Successfully built get_balance trusted operation, dispatching now to enclave");

    // let bal = if let Some(v) = perform_operation(matches, &get_balance_top) {
    //     if let Ok(vd) = Balance::decode(&mut v.as_slice()) {
    //         vd
    //     } else {
    //         info!("could not decode value. maybe hasn't been set? {:x?}", v);
    //         0
    //     }
    // } else {
    //     0
    // };
    // println!("{}", bal);
    let bal = if let Some(v) = perform_operation(matches, &get_balance_top) {
        if let Ok(vd) = Balances::decode(&mut v.as_slice()) {
            vd
        } else {
            info!("could not decode value. maybe hasn't been set? {:x?}", v);
            Balances::from(0, 0)
        }
    } else {
        Balances::from(0, 0)
    };
    error!("Res Bal {:?}", bal.reserved);
    println!("{}", bal.free);

    Ok(())
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Balances {
    pub free: Balance,
    pub reserved: Balance,
}

impl Balances {
    pub fn from(free: Balance, reserved: Balance) -> Self {
        Self { free, reserved }
    }
}
