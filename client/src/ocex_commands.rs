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

use log::*;

use clap::{App, Arg, ArgMatches};
use clap_nested::Command;
use my_node_runtime::AccountId;
use polkadex_sgx_primitives::{AssetId, Balance};
use sp_core::{sr25519 as sr25519_core, Pair};
use substrate_api_client::{
    compose_extrinsic, extrinsic::xt_primitives::UncheckedExtrinsicV4, XtStatus,
};

use substratee_stf::commands::{common_args, common_args_processing};

pub fn register_account_command<'a>() -> Command<'a, str> {
    Command::new("register-account")
        .description("Registers a new main account to the polkadex offchain registry")
        .options(|app| {
            app.arg(
                Arg::with_name("main")
                    .takes_value(true)
                    .required(true)
                    .value_name("SS58")
                    .help("Sender's on-chain AccountId in ss58check format"),
            )
        })
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            let chain_api = crate::get_chain_api(matches);

            // get the main account / sender
            let arg_main = matches.value_of("main").unwrap();
            let main = crate::get_pair_from_str_untrusted(arg_main);
            let account_id = sr25519_core::Pair::from(main);
            let chain_api = chain_api.set_signer(account_id.clone());

            // compose the extrinsic
            let xt: UncheckedExtrinsicV4<([u8; 2], AccountId)> = compose_extrinsic!(
                chain_api,
                "PolkadexOcex",
                "register",
                account_id.public().into()
            );

            let tx_hash =
                chain_api.send_extrinsic(xt.hex_encode(), XtStatus::Finalized).unwrap().unwrap();
            println!("[+] Transaction got finalized.. Hash: {:?}\n", tx_hash);
            Ok(())
        })
}

pub fn register_proxy_command<'a>() -> Command<'a, str> {
    Command::new("register-proxy")
        .description("Registers a new proxy account to the polkadex offchain registry")
        .options(|app| {
            app.arg(
                Arg::with_name("main")
                    .takes_value(true)
                    .required(true)
                    .value_name("SS58")
                    .help("Sender's on-chain AccountId in ss58check format"),
            )
            .arg(
                Arg::with_name("proxy")
                    .takes_value(true)
                    .required(true)
                    .value_name("SS58")
                    .help("Sender's proxy AccountId in ss58check format"),
            )
        })
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            let chain_api = crate::get_chain_api(matches);

            // get the main account /sender
            let arg_main = matches.value_of("main").unwrap();
            let main = crate::get_pair_from_str_untrusted(arg_main);
            let main_account_id = sr25519_core::Pair::from(main);
            let chain_api = chain_api.set_signer(main_account_id.clone());

            // get the proxy account
            let proxy = crate::get_accountid_from_str(matches.value_of("proxy").unwrap());

            // compose the extrinsic
            let xt: UncheckedExtrinsicV4<([u8; 2], AccountId, AccountId)> = compose_extrinsic!(
                chain_api,
                "PolkadexOcex",
                "add_proxy",
                main_account_id.public().into(),
                proxy
            );

            let tx_hash =
                chain_api.send_extrinsic(xt.hex_encode(), XtStatus::Finalized).unwrap().unwrap();
            println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
            Ok(())
        })
}

pub fn remove_proxy_command<'a>() -> Command<'a, str> {
    Command::new("remove-proxy")
        .description("Removes a registered proxy account from the polkadex offchain registry")
        .options(|app| {
            app.arg(
                Arg::with_name("main")
                    .takes_value(true)
                    .required(true)
                    .value_name("SS58")
                    .help("Sender's on-chain AccountId in ss58check format"),
            )
            .arg(
                Arg::with_name("proxy")
                    .takes_value(true)
                    .required(true)
                    .value_name("SS58")
                    .help("Sender's proxy AccountId in ss58check format"),
            )
        })
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            let chain_api = crate::get_chain_api(matches);

            // get the main account /sender
            let arg_main = matches.value_of("main").unwrap();
            let main = crate::get_pair_from_str_untrusted(arg_main);
            let main_account_id = sr25519_core::Pair::from(main);
            let chain_api = chain_api.set_signer(main_account_id.clone());

            // get the proxy account
            let proxy = crate::get_accountid_from_str(matches.value_of("proxy").unwrap());

            // compose the extrinsic
            let xt: UncheckedExtrinsicV4<([u8; 2], AccountId, AccountId)> = compose_extrinsic!(
                chain_api,
                "PolkadexOcex",
                "remove_proxy",
                main_account_id.public().into(),
                proxy
            );

            let tx_hash =
                chain_api.send_extrinsic(xt.hex_encode(), XtStatus::Finalized).unwrap().unwrap();
            println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
            Ok(())
        })
}

pub fn withdraw_command<'a>() -> Command<'a, str> {
    Command::new("withdraw")
		.description("withdraws the given amount of funds from the sender")
		.options(add_command_args)
		.runner(move |_args: &str, matches: &ArgMatches<'_>| {
			let chain_api = crate::get_chain_api(matches);

			// get the main account /sender
			let arg_main = matches.value_of("accountid").unwrap();
			let main = crate::get_pair_from_str_untrusted(arg_main);
			let main_account_id = sr25519_core::Pair::from(main);
			let chain_api = chain_api.set_signer(main_account_id.clone());

			let asset_id = common_args_processing::get_token_id_from_matches(matches).unwrap();
			let amount = common_args_processing::get_quantity_from_matches(matches).unwrap();

			// compose the extrinsic
			let xt: UncheckedExtrinsicV4<([u8; 2], AccountId, AssetId, Balance)> = compose_extrinsic!(
				chain_api,
				"PolkadexOcex",
				"withdraw",
				main_account_id.public().into(),
				asset_id,
				amount
			);

			let tx_hash =
				chain_api.send_extrinsic(xt.hex_encode(), XtStatus::Finalized).unwrap().unwrap();
			println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
			Ok(())
		})
}

fn add_command_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let app_with_main_account = common_args::add_main_account_args(app);
    let app_with_token_id = common_args::add_token_id_args(app_with_main_account);
    common_args::add_quantity_args(app_with_token_id)
}

pub fn deposit_command<'a>() -> Command<'a, str> {
    Command::new("deposit")
		.description("deposits a given amount of funds from the sender")
		.options(add_command_args)
		.runner(move |_args: &str, matches: &ArgMatches<'_>| {
			let chain_api = crate::get_chain_api(matches);

			// get the main account /sender
			let arg_main = matches.value_of("accountid").unwrap();
			let main = crate::get_pair_from_str_untrusted(arg_main);
			let main_account_id = sr25519_core::Pair::from(main);
			let chain_api = chain_api.set_signer(main_account_id.clone());

			let asset_id = common_args_processing::get_token_id_from_matches(matches).unwrap();
			let amount = common_args_processing::get_quantity_from_matches(matches).unwrap();

			// compose the extrinsic
			let xt: UncheckedExtrinsicV4<([u8; 2], AccountId, AssetId, Balance)> = compose_extrinsic!(
				chain_api,
				"PolkadexOcex",
				"deposit",
				main_account_id.public().into(),
				asset_id,
				amount
			);

			let tx_hash =
				chain_api.send_extrinsic(xt.hex_encode(), XtStatus::Finalized).unwrap().unwrap();
			println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
			Ok(())
		})
}
