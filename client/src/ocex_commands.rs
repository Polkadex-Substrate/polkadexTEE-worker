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
                    Arg::with_name("from")
                        .takes_value(true)
                        .required(true)
                        .value_name("SS58")
                        .help("Sender's on-chain AccountId in ss58check format"),
                )
            })
            .runner(move |_args: &str, matches: &ArgMatches<'_>| {
                let chain_api = crate::get_chain_api(matches);

                // get the sender
                let arg_from = matches.value_of("from").unwrap();
                let from = crate::get_pair_from_str(matches, arg_from);
                let account_id = sr25519_core::Pair::from(from);
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
                    .unwrap();
                println!("[+] TrustedOperation got finalized. Hash: {:?}\n", tx_hash);
                Ok(())
            })
}
