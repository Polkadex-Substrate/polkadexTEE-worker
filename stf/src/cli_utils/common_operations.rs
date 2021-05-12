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

use crate::cli_utils::common_types::OperationRunner;
use crate::{Index, KeyPair, TrustedGetter, TrustedOperation};

use clap::ArgMatches;
use codec::Decode;
use log::*;
use sp_application_crypto::sr25519;
use sp_core::{sr25519 as sr25519_core, Pair};

pub fn get_trusted_nonce(
    perform_operation: OperationRunner<'_>,
    matches: &ArgMatches,
    who: &sr25519::AppPair,
    key_pair: &sr25519_core::Pair,
) -> u32 {
    let top: TrustedOperation =
        TrustedGetter::nonce(sr25519_core::Public::from(who.public()).into())
            .sign(&KeyPair::Sr25519(key_pair.clone()))
            .into();
    let res = perform_operation(matches, &top);
    let nonce: Index = if let Some(n) = res {
        if let Ok(nonce) = Index::decode(&mut n.as_slice()) {
            nonce
        } else {
            info!("could not decode value. maybe hasn't been set? {:x?}", n);
            0
        }
    } else {
        0
    };
    debug!("got nonce: {:?}", nonce);
    nonce
}
