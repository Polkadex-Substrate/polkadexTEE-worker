// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub extern crate alloc;
use alloc::string::String;

use base58::ToBase58;
use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
use sp_core::crypto::{AccountId32, Ss58AddressFormat};

/// utility function to get the ss58check string representation for an AccountId32
///
pub fn account_id_to_ss58check(account_id: &AccountId32) -> String {
    to_ss58check(account_id)
}

/// This below is copied code from substrate crypto in order to make the ss58check
/// utility functions available inside the enclave.
/// This is a quick workaround done now for the PolkaDex POC
/// We should probably find a better solution, like forking the repo or upstreaming
/// the changes to remove the 'std' feature guards in the original code

/// Return the ss58-check string for this key.
fn to_ss58check(account_id: &AccountId32) -> String {
    to_ss58check_with_version(account_id, Ss58AddressFormat::SubstrateAccount)
}

/// Return the ss58-check string for this key.
fn to_ss58check_with_version(account_id: &AccountId32, version: Ss58AddressFormat) -> String {
    // We mask out the upper two bits of the ident - SS58 Prefix currently only supports 14-bits
    let ident: u16 = u16::from(version) & 0b00111111_11111111;
    let mut v = match ident {
        0..=63 => vec![ident as u8],
        64..=16_383 => {
            // upper six bits of the lower byte(!)
            let first = ((ident & 0b00000000_11111100) as u8) >> 2;
            // lower two bits of the lower byte in the high pos,
            // lower bits of the upper byte in the low pos
            let second = ((ident >> 8) as u8) | ((ident & 0b00000000_00000011) as u8) << 6;
            vec![first | 0b01000000, second]
        }
        _ => unreachable!("masked out the upper two bits; qed"),
    };

    let account_id_arr: [u8; 32] = account_id.clone().into();

    v.extend(account_id_arr.as_ref());
    let r = ss58hash(&v);
    v.extend(&r.as_bytes()[0..2]);
    v.to_base58()
}

const PREFIX: &[u8] = b"SS58PRE";

fn ss58hash(data: &[u8]) -> Blake2bResult {
    let mut context = Blake2b::new(64);
    context.update(PREFIX);
    context.update(data);
    context.finalize()
}
