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

use crate::{
    AccountId, Index, KeyPair, ShardIdentifier, TrustedCall, TrustedGetter, TrustedOperation,
};
use base58::{FromBase58, ToBase58};
use clap::{AppSettings, Arg, ArgMatches};
use clap_nested::{Command, Commander, MultiCommand};
use codec::{Decode, Encode};
use log::*;
use sp_application_crypto::{ed25519, sr25519};
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair};
use sp_runtime::traits::IdentifyAccount;
use std::path::PathBuf;
use substrate_client_keystore::{KeystoreExt, LocalKeystore};

use crate::{
    cli_utils::{
        account_parsing::*, common_operations::get_trusted_nonce, common_types::OperationRunner,
    },
    commands::{
        cancel_order::cancel_order_cli_command, get_balance::get_balance_cli_command,
        place_order::place_order_cli_command, withdraw::withdraw_cli_command,
    },
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn cmd(perform_operation: OperationRunner) -> MultiCommand<str, str> {
    Commander::new()
        .options(|app| {
            app.setting(AppSettings::ColoredHelp)
                .arg(
                    Arg::with_name("mrenclave")
                        .short("m")
                        .long("mrenclave")
                        .global(true)
                        .takes_value(true)
                        .value_name("STRING")
                        .help("targeted worker MRENCLAVE"),
                )
                .arg(
                    Arg::with_name("shard")
                        .short("s")
                        .long("shard")
                        .global(true)
                        .takes_value(true)
                        .value_name("STRING")
                        .help("shard identifier"),
                )
                .arg(
                    Arg::with_name("xt-signer")
                        .short("a")
                        .long("xt-signer")
                        .global(true)
                        .takes_value(true)
                        .value_name("AccountId")
                        .default_value("//Alice")
                        .help("signer for publicly observable extrinsic"),
                )
                .arg(
                    Arg::with_name("direct")
                        .short("d")
                        .long("direct")
                        .global(true)
                        .help("insert if direct invocation call is desired"),
                )
                .name("substratee-client")
                .version(VERSION)
                .author("Supercomputing Systems AG <info@scs.ch>")
                .about("trusted calls to worker enclave")
                .after_help("stf subcommands depend on the stf crate this has been built against")
        })
        .add_cmd(place_order_cli_command(perform_operation))
        .add_cmd(cancel_order_cli_command(perform_operation))
        .add_cmd(get_balance_cli_command(perform_operation))
        .add_cmd(withdraw_cli_command(perform_operation))
        .add_cmd(
            Command::new("new-account")
                .description("generates a new incognito account for the given substraTEE shard")
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    let store =
                        LocalKeystore::open(get_trusted_keystore_path(matches), None).unwrap();
                    let key: sr25519::AppPair = store.generate().unwrap();
                    drop(store);
                    println!("{}", key.public().to_ss58check());
                    Ok(())
                }),
        )
        .add_cmd(
            Command::new("list-accounts")
                .description("lists all accounts in keystore for the substraTEE chain")
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    let store =
                        LocalKeystore::open(get_trusted_keystore_path(matches), None).unwrap();
                    info!("sr25519 keys:");
                    for pubkey in store
                        .public_keys::<sr25519::AppPublic>()
                        .unwrap()
                        .into_iter()
                    {
                        println!("{}", pubkey.to_ss58check());
                    }
                    info!("ed25519 keys:");
                    for pubkey in store
                        .public_keys::<ed25519::AppPublic>()
                        .unwrap()
                        .into_iter()
                    {
                        println!("{}", pubkey.to_ss58check());
                    }
                    drop(store);
                    Ok(())
                }),
        )
        .add_cmd(
            Command::new("transfer")
                .description("send funds from one incognito account to another")
                .options(|app| {
                    app.setting(AppSettings::ColoredHelp)
                        .arg(
                            Arg::with_name("from")
                                .takes_value(true)
                                .required(true)
                                .value_name("SS58")
                                .help("sender's AccountId in ss58check format"),
                        )
                        .arg(
                            Arg::with_name("to")
                                .takes_value(true)
                                .required(true)
                                .value_name("SS58")
                                .help("recipient's AccountId in ss58check format"),
                        )
                        .arg(
                            Arg::with_name("amount")
                                .takes_value(true)
                                .required(true)
                                .value_name("U128")
                                .help("amount to be transferred"),
                        )
                })
                .runner(move |_args: &str, matches: &ArgMatches<'_>| {
                    let arg_from = matches.value_of("from").unwrap();
                    let arg_to = matches.value_of("to").unwrap();
                    let amount = matches
                        .value_of("amount")
                        .unwrap()
                        .parse::<u128>()
                        .expect("amount can be converted to u128");
                    let from = get_pair_from_str_trusted(matches, arg_from);
                    let to = get_accountid_from_str(arg_to);
                    let direct: bool = matches.is_present("direct");
                    info!("from ss58 is {}", from.public().to_ss58check());
                    info!("to ss58 is {}", to.to_ss58check());

                    println!(
                        "send trusted call transfer from {} to {}: {}",
                        from.public(),
                        to,
                        amount
                    );
                    let (mrenclave, shard) = get_identifiers(matches);
                    let key_pair = sr25519_core::Pair::from(from.clone());
                    let nonce = get_trusted_nonce(perform_operation, matches, &from, &key_pair);

                    let top: TrustedOperation = TrustedCall::balance_transfer(
                        sr25519_core::Public::from(from.public()).into(),
                        to,
                        amount,
                    )
                    .sign(&KeyPair::Sr25519(key_pair), nonce, &mrenclave, &shard)
                    .into_trusted_operation(direct);
                    let _ = perform_operation(matches, &top);
                    Ok(())
                }),
        )
        .add_cmd(
            Command::new("set-balance")
                .description("ROOT call to set some account balance to an arbitrary number")
                .options(|app| {
                    app.arg(
                        Arg::with_name("account")
                            .takes_value(true)
                            .required(true)
                            .value_name("SS58")
                            .help("sender's AccountId in ss58check format"),
                    )
                    .arg(
                        Arg::with_name("amount")
                            .takes_value(true)
                            .required(true)
                            .value_name("U128")
                            .help("amount to be transferred"),
                    )
                })
                .runner(move |_args: &str, matches: &ArgMatches<'_>| {
                    let arg_who = matches.value_of("account").unwrap();
                    let amount = matches
                        .value_of("amount")
                        .unwrap()
                        .parse::<u128>()
                        .expect("amount can be converted to u128");
                    let who = get_pair_from_str_trusted(matches, arg_who);
                    let signer = get_pair_from_str_trusted(matches, "//Alice");
                    let direct: bool = matches.is_present("direct");
                    info!("account ss58 is {}", who.public().to_ss58check());

                    println!(
                        "send trusted call set-balance({}, {})",
                        who.public(),
                        amount
                    );

                    let (mrenclave, shard) = get_identifiers(matches);
                    let key_pair = sr25519_core::Pair::from(who.clone());
                    let nonce = get_trusted_nonce(perform_operation, matches, &who, &key_pair);

                    let top: TrustedOperation = TrustedCall::balance_set_balance(
                        sr25519_core::Public::from(signer.public()).into(),
                        sr25519_core::Public::from(who.public()).into(),
                        amount,
                        amount,
                    )
                    .sign(&KeyPair::Sr25519(key_pair), nonce, &mrenclave, &shard)
                    .into_trusted_operation(direct);
                    let _ = perform_operation(matches, &top);
                    Ok(())
                }),
        )
        .add_cmd(
            Command::new("balance")
                .description("query balance for incognito account in keystore")
                .options(|app| {
                    app.arg(
                        Arg::with_name("accountid")
                            .takes_value(true)
                            .required(true)
                            .value_name("SS58")
                            .help("AccountId in ss58check format"),
                    )
                })
                .runner(move |_args: &str, matches: &ArgMatches<'_>| {
                    let arg_who = matches.value_of("accountid").unwrap();
                    info!("arg_who = {:?}", arg_who);
                    let who = get_pair_from_str_trusted(matches, arg_who);
                    let key_pair = sr25519_core::Pair::from(who.clone());
                    let top: TrustedOperation = TrustedGetter::free_balance(
                        sr25519_core::Public::from(who.public()).into(),
                    )
                    .sign(&KeyPair::Sr25519(key_pair))
                    .into();
                    let res = perform_operation(matches, &top);
                    let bal = if let Some(v) = res {
                        if let Ok(vd) = crate::Balance::decode(&mut v.as_slice()) {
                            vd
                        } else {
                            info!("could not decode value. maybe hasn't been set? {:x?}", v);
                            0
                        }
                    } else {
                        0
                    };
                    println!("{}", bal);
                    Ok(())
                }),
        )
        .add_cmd(
            Command::new("unshield-funds")
                .description("Transfer funds from an incognito account to an on-chain account")
                .options(|app| {
                    app.arg(
                        Arg::with_name("from")
                            .takes_value(true)
                            .required(true)
                            .value_name("SS58")
                            .help("Sender's incognito AccountId in ss58check format"),
                    )
                    .arg(
                        Arg::with_name("to")
                            .takes_value(true)
                            .required(true)
                            .value_name("SS58")
                            .help("Recipient's on-chain AccountId in ss58check format"),
                    )
                    .arg(
                        Arg::with_name("amount")
                            .takes_value(true)
                            .required(true)
                            .value_name("U128")
                            .help("Amount to be transferred"),
                    )
                    .arg(
                        Arg::with_name("shard")
                            .takes_value(true)
                            .required(true)
                            .value_name("STRING")
                            .help("Shard identifier"),
                    )
                })
                .runner(move |_args: &str, matches: &ArgMatches<'_>| {
                    let arg_from = matches.value_of("from").unwrap();
                    let arg_to = matches.value_of("to").unwrap();
                    let amount = matches
                        .value_of("amount")
                        .unwrap()
                        .parse::<u128>()
                        .expect("amount can be converted to u128");
                    let from = get_pair_from_str_trusted(matches, arg_from);
                    let to = get_accountid_from_str(arg_to);
                    let direct: bool = matches.is_present("direct");
                    println!("from ss58 is {}", from.public().to_ss58check());
                    println!("to   ss58 is {}", to.to_ss58check());

                    println!(
                        "send trusted call unshield_funds from {} to {}: {}",
                        from.public(),
                        to,
                        amount
                    );

                    let (mrenclave, shard) = get_identifiers(matches);
                    let key_pair = sr25519_core::Pair::from(from.clone());
                    let nonce = get_trusted_nonce(perform_operation, matches, &from, &key_pair);

                    let top: TrustedOperation = TrustedCall::balance_unshield(
                        sr25519_core::Public::from(from.public()).into(),
                        to,
                        amount,
                        shard,
                    )
                    .sign(&KeyPair::Sr25519(key_pair), nonce, &mrenclave, &shard)
                    .into_trusted_operation(direct);
                    let _ = perform_operation(matches, &top);
                    Ok(())
                }),
        )
        .into_cmd("trusted")
}
