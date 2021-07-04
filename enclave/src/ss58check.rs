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
use alloc::{fmt::Display, fmt::Formatter, fmt::Result as FormatResult, string::String};
use base58::{FromBase58, ToBase58};
use blake2_rfc::blake2b::{Blake2b, Blake2bResult};
use core::convert::AsMut;
use sp_core::crypto::{AccountId32, Ss58AddressFormat};
use sp_core::sp_std::convert::TryInto;
use log::*;

/// errors related to the conversion to/from SS58Check format
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum SS58CheckError {
    BadBase58,
    BadLength,
    UnknownVersion,
    FormatNotAllowed,
    InvalidChecksum,
}

impl Display for SS58CheckError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        write!(f, "{:?}", self)
    }
}

/// utility function to get the ss58check string representation for an AccountId32
pub fn account_id_to_ss58check(account_id: &AccountId32) -> String {
    to_ss58check(account_id)
}

/// utility function to get the account ID, back from a ss58check string
pub fn ss58check_to_account_id(s: &str) -> Result<AccountId32, SS58CheckError> {
    from_ss58check_with_version(s).map(|t| t.0)
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

/// Some if the string is a properly encoded SS58Check address.
fn from_ss58check_with_version(
    s: &str,
) -> Result<(AccountId32, Ss58AddressFormat), SS58CheckError> {
    debug!("Decoding: {}", s);
    const CHECKSUM_LEN: usize = 2;
    let mut res = AccountId32::default();

    // Must decode to our type.
    let tmp_arr: &mut [u8] = res.as_mut();
    let body_len = tmp_arr.len();

    let data = s.from_base58().map_err(|_| SS58CheckError::BadBase58)?;
    if data.len() < 2 {
        return Err(SS58CheckError::BadLength);
    }
    let (prefix_len, ident) = match data[0] {
        0..=63 => (1, data[0] as u16),
        64..=127 => {
            // weird bit manipulation owing to the combination of LE encoding and missing two bits
            // from the left.
            // d[0] d[1] are: 01aaaaaa bbcccccc
            // they make the LE-encoded 16-bit value: aaaaaabb 00cccccc
            // so the lower byte is formed of aaaaaabb and the higher byte is 00cccccc
            let lower = (data[0] << 2) | (data[1] >> 6);
            let upper = data[1] & 0b00111111;
            (2, (lower as u16) | ((upper as u16) << 8))
        }
        _ => Err(SS58CheckError::UnknownVersion)?,
    };
    if data.len() != prefix_len + body_len + CHECKSUM_LEN {
        return Err(SS58CheckError::BadLength);
    }

    let format = ident
        .try_into()
        .map_err(|_: ()| SS58CheckError::UnknownVersion)?;
    if !format_is_allowed(format) {
        return Err(SS58CheckError::FormatNotAllowed);
    }

    let hash = ss58hash(&data[0..body_len + prefix_len]);
    let checksum = &hash.as_bytes()[0..CHECKSUM_LEN];
    if data[body_len + prefix_len..body_len + prefix_len + CHECKSUM_LEN] != *checksum {
        // Invalid checksum.
        return Err(SS58CheckError::InvalidChecksum);
    }
    tmp_arr
        .as_mut()
        .copy_from_slice(&data[prefix_len..body_len + prefix_len]);
    Ok((res, format))
}

/// A format filterer, can be used to ensure that `from_ss58check` family only decode for
/// allowed identifiers. By default just refuses the two reserved identifiers.
fn format_is_allowed(f: Ss58AddressFormat) -> bool {
    !matches!(
        f,
        Ss58AddressFormat::Reserved46 | Ss58AddressFormat::Reserved47
    )
}

pub mod tests {

    use super::*;
    use sp_core::{ed25519 as ed25519_core, Pair};

    pub fn convert_account_id_to_and_from_ss58check() {
        let account_key = ed25519_core::Pair::from_seed(b"12345678901234567890123456789012");
        let account_id: AccountId32 = account_key.public().into();

        let ss58check_str = to_ss58check(&account_id);
        let mapped_account_id = from_ss58check_with_version(ss58check_str.as_str()).unwrap();

        assert_eq!(mapped_account_id.0, account_id);
    }
}
