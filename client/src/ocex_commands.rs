use log::*;

use base58::{FromBase58, ToBase58};
use clap::{AppSettings, Arg, ArgMatches};
use clap_nested::{Command, Commander};
use codec::{Decode, Encode};
use log::*;
use my_node_runtime::{
    pallet_substratee_registry::{Enclave, Request},
    AccountId, BalancesCall, Call, Event, Hash,
};
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
                let public_account_id: AccountId = account_id.public().into();

                // compose the extrinsic
                let xt: UncheckedExtrinsicV4<([u8; 2], AccountId)> = compose_extrinsic!(
                    chain_api,
                    "PolkadexOcex",
                    "register",
                    public_account_id.clone()
                );

                let tx_hash = chain_api
                    .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
                    .unwrap()
                    .unwrap();
                println!("[+] Successfully registered new account {}. Hash: {:?}\n", public_account_id, tx_hash);
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
                proxy.clone()
            );

            let tx_hash = chain_api
                .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
                .unwrap()
                .unwrap();
            println!("[+] Successfully registered new proxy account: {}. Hash: {:?}\n", proxy, tx_hash);
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
                proxy.clone()
            );

            let tx_hash = chain_api
                .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
                .unwrap()
                .unwrap();
            println!("[+] Successfully removed proxy account: {}. Hash: {:?}\n", proxy,  tx_hash);
            Ok(())
        })
}
