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

use crate::{AccountId, ShardIdentifier};

use base58::{FromBase58, ToBase58};
use clap::ArgMatches;
use codec::Encode;
use log::*;
use sp_application_crypto::sr25519;
use sp_core::{crypto::Ss58Codec, Pair};
use sp_runtime::traits::IdentifyAccount;
use std::path::PathBuf;
use substrate_client_keystore::LocalKeystore;

const KEYSTORE_PATH: &str = "my_trusted_keystore";

pub fn get_accountid_from_str(account: &str) -> AccountId {
    match &account[..2] {
        "//" => sr25519::Pair::from_string(account, None)
            .unwrap()
            .public()
            .into_account()
            .into(),
        _ => sr25519::Public::from_ss58check(account)
            .unwrap()
            .into_account()
            .into(),
    }
}

// get a pair either form keyring (well known keys) or from the store
pub fn get_pair_from_str(matches: &ArgMatches<'_>, account: &str) -> sr25519::AppPair {
    info!("getting pair for {}", account);
    match &account[..2] {
        "//" => sr25519::AppPair::from_string(account, None).unwrap(),
        _ => {
            info!("fetching from keystore at {}", &KEYSTORE_PATH);
            // open store without password protection
            let store =
                LocalKeystore::open(get_keystore_path(matches), None).expect("store should exist");
            info!("store opened");
            let _pair = store
                .key_pair::<sr25519::AppPair>(
                    &sr25519::Public::from_ss58check(account).unwrap().into(),
                )
                .unwrap()
                .unwrap();
            info!("key pair fetched");
            drop(store);
            _pair
        }
    }
}

pub fn get_keystore_path(matches: &ArgMatches<'_>) -> PathBuf {
    let (_mrenclave, shard) = get_identifiers(matches);
    PathBuf::from(&format!("{}/{}", KEYSTORE_PATH, shard.encode().to_base58()))
}

pub fn get_identifiers(matches: &ArgMatches<'_>) -> ([u8; 32], ShardIdentifier) {
    let mut mrenclave = [0u8; 32];
    if !matches.is_present("mrenclave") {
        panic!("--mrenclave must be provided");
    };
    mrenclave.copy_from_slice(
        &matches
            .value_of("mrenclave")
            .unwrap()
            .from_base58()
            .expect("mrenclave has to be base58 encoded"),
    );
    let shard = match matches.value_of("shard") {
        Some(val) => {
            ShardIdentifier::from_slice(&val.from_base58().expect("shard has to be base58 encoded"))
        }
        None => ShardIdentifier::from_slice(&mrenclave),
    };
    (mrenclave, shard)
}
