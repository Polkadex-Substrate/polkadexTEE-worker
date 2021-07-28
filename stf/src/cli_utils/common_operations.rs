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

use crate::cli_utils::common_types::OperationRunner;
use crate::Index;
use crate::{KeyPair, TrustedGetter, TrustedOperation};

use clap::ArgMatches;
use codec::Decode;
use log::*;
use sp_application_crypto::sr25519;
use sp_core::sr25519 as sr25519_core;
use sp_core::Pair;

pub fn get_trusted_nonce(
    perform_operation: OperationRunner<'_>,
    matches: &ArgMatches,
    who: &sr25519::AppPair,
    key_pair: &sr25519_core::Pair,
) -> Index {
    let top: TrustedOperation =
        TrustedGetter::nonce(sr25519_core::Public::from(who.public()).into())
            .sign(&KeyPair::Sr25519(key_pair.clone()))
            .into();
    let res = perform_operation(matches, &top);
    let nonce: Index = if let Some(n) = res {
        if let Ok(nonce) = Index::decode(&mut n.as_slice()) {
            nonce
        } else {
            error!("could not decode value. maybe hasn't been set? {:x?}", n);
            0
        }
    } else {
        0
    };
    debug!("got nonce: {:?}", nonce);
    nonce
}
