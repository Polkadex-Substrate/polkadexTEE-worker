#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

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

pub mod accounts;
pub mod types;

pub extern crate alloc;
use alloc::string::{String, ToString};

use codec::{Decode, Encode};
use frame_support::{sp_runtime::traits::AccountIdConversion, PalletId};
pub use polkadex_primitives::assets::AssetId;
pub use polkadex_primitives::common_types::{AccountId, Balance, Signature};
use sp_core::H256;
use sp_std::vec::Vec;

pub type ShardIdentifier = H256;
pub type BlockNumber = u32;

// Genesis Account constant should be kept up to date with OCEXGenesisAccount at https://github.com/Polkadex-Substrate/Polkadex/blob/main/runtime/src/lib.rs#L1536
const GENESIS_ACCOUNT: PalletId = PalletId(*b"polka/ga");

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct LinkedAccount {
    pub prev: AccountId,
    pub current: AccountId,
    pub next: Option<AccountId>,
    pub proxies: Vec<AccountId>,
}

impl LinkedAccount {
    pub fn from(prev: AccountId, current: AccountId) -> Self {
        LinkedAccount {
            prev,
            next: None,
            current,
            proxies: vec![],
        }
    }
}

impl Default for LinkedAccount {
    fn default() -> Self {
        LinkedAccount {
            prev: GENESIS_ACCOUNT.into_account(),
            current: GENESIS_ACCOUNT.into_account(),
            next: None,
            proxies: vec![],
        }
    }
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct PolkadexAccount {
    pub account: LinkedAccount,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct OpenFinexUri {
    ip: Vec<u8>,
    port: u16,
    path: Vec<u8>,
}

impl OpenFinexUri {
    /// creates a new openfinex uri used to connect to openfinex server.
    /// INPUT:  ip_str (e.g. "localhost" or "127.0.0.1")
    ///         port_path_str, string containing at least the port
    ///             (e.g.: 8001/api/v2/ws or 8001)
    pub fn new(ip_str: &str, port_path_str: &str) -> Self {
        let (port_str, path_str) = if let Some(index) = port_path_str.find('/') {
            port_path_str.split_at(index)
        } else {
            (port_path_str, "/")
        };
        OpenFinexUri {
            ip: ip_str.as_bytes().to_vec(),
            port: port_str.parse().unwrap(),
            path: path_str.as_bytes().to_vec(),
        }
    }

    //FIXME: we should use &str or anything similar here..
    pub fn ip(&self) -> String {
        String::from_utf8_lossy(&self.ip).into_owned()
    }

    pub fn port(&self) -> String {
        self.port.to_string()
    }

    // FIXME: maybe port should be a u XX right away?
    pub fn port_u16(&self) -> u16 {
        self.port
    }

    pub fn path(&self) -> String {
        String::from_utf8_lossy(&self.path).into_owned()
    }
}

// TODO: Add unit tests
