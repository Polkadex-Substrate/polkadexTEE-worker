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

use polkadex_sgx_primitives::AccountId;
use log::*;
use codec::Encode;
use sgx_tstd::collections::HashMap;
use sgx_tstd::vec::Vec;

pub type EncodedKey = Vec<u8>;

#[derive(Debug)]
pub struct PolkadexNonceStorage {
    /// map AccountId -> NonceHandler
    pub storage: HashMap<EncodedKey, u32>,
}

impl PolkadexNonceStorage {
    pub fn create() -> PolkadexNonceStorage {
        PolkadexNonceStorage {
            storage: HashMap::new(),
        }
    }

    pub fn read_nonce(&mut self, acc: AccountId) -> u32 {
        debug!("reading nonce from acc: {:?}", acc);
        if let Some(nonce) = self.storage.get(&acc.clone().encode()) {
            *nonce
        }
        else {
            self.initialize_nonce(acc);
            0u32
        }
    }

    pub fn set_nonce(&mut self, nonce: u32, acc: AccountId) {
        self
            .storage
            .insert(acc.clone().encode(), nonce);
    }

    pub fn increment_nonce(&mut self, acc: AccountId) {
        let nonce = self.read_nonce(acc.clone());
        self
            .storage
            .insert(acc.clone().encode(), nonce + 1u32);
    }

    pub fn initialize_nonce(&mut self, acc: AccountId) {
        debug!("initializing nonce for acc: {:?}", acc);
        self.storage.insert(acc.encode(), 0u32);
    }
}