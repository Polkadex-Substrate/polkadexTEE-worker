use log::*;

use base58::{FromBase58, ToBase58};
use clap::{AppSettings, Arg, ArgMatches, App};
use clap_nested::{Command, Commander};
use codec::{Decode, Encode};
use log::*;
use my_node_runtime::{
    pallet_substratee_registry::{Enclave, Request},
    AccountId, BalancesCall, Call, Event, Hash,
};
use polkadex_sgx_primitives::{AssetId, PolkadexAccount, Balance};
use polkadex_sgx_primitives::types::DirectRequest;
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use sp_application_crypto::{ed25519, sr25519};
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair, H256};
use sp_keyring::AccountKeyring;
use sp_runtime::MultiSignature;
use std::convert::TryFrom;
use std::result::Result as StdResult;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, UNIX_EPOCH};
use substrate_api_client::{
    compose_extrinsic, compose_extrinsic_offline,
    events::EventsDecoder,
    extrinsic::xt_primitives::{GenericAddress, UncheckedExtrinsicV4},
    node_metadata::Metadata,
    utils::FromHexString,
    Api, XtStatus,
};

use substratee_stf::commands;
use substratee_stf::commands::{common_args_processing, common_args};

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
                let main = crate::get_pair_from_str(matches, arg_main);
                let account_id = sr25519_core::Pair::from(main);
                let chain_api = chain_api.set_signer(account_id.clone());

                // compose the extrinsic
                let xt: UncheckedExtrinsicV4<([u8; 2], AccountId)> = compose_extrinsic!(
                    chain_api,
                    "PolkadexOcex",
                    "register",
                    account_id.public().into()
                );

                let tx_hash = chain_api
                    .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
                    .unwrap()
                    .unwrap();
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
            let main = crate::get_pair_from_str(matches, arg_main);
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

            let tx_hash = chain_api
                .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
                .unwrap()
                .unwrap();
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
            let main = crate::get_pair_from_str(matches, arg_main);
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

            let tx_hash = chain_api
                .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
                .unwrap()
                .unwrap();
            println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
            Ok(())
        })
}

pub fn withdraw_command<'a>() -> Command<'a, str> {
    Command::new("withdraw")
        .description("withdraws the given amount of funds from the sender")
        .options(|app| add_withdraw_command_args(app))
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            let chain_api = crate::get_chain_api(matches);

            // get the main account /sender
            let arg_main = matches.value_of("main").unwrap();
            let main = crate::get_pair_from_str(matches, arg_main);
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

            let tx_hash = chain_api
                .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
                .unwrap()
                .unwrap();
            println!("[+] Transaction got finalized.  Hash: {:?}\n",  tx_hash);
            Ok(())
        })
}

fn add_withdraw_command_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let app_with_main_account = common_args::add_main_account_args(app);
    let app_with_token_id = common_args::add_token_id_args(app_with_main_account);
    common_args::add_quantity_args(app_with_token_id)
}
