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


use crate::polkadex_nonce_storage::nonce_handler::*;

pub type EncodedKey = Vec<u8>;

#[derive(Debug)]
pub struct PolkadexNonceStorage {
    /// map AccountId -> NonceHandler
    pub storage: HashMap<EncodedKey, NonceHandler>,
}

impl PolkadexNonceStorage {
    pub fn create() -> PolkadexNonceStorage {
        PolkadexNonceStorage {
            storage: HashMap::new(),
        }
    }

    pub fn read_nonce(&self, acc: AccountId) -> Option<&NonceHandler> {
        debug!("reading nonce from acc: {:?}", acc);
        self.storage.get(&acc.encode())
    }

    pub fn set_nonce(&mut self, nonce: u32, acc: AccountId) {
        match self
            .storage
            .get_mut(&acc.clone().encode())
        {
            Some(nonce_handler) => {
                nonce_handler.nonce = Some(nonce);
            }
            None => {
                debug!("No entry available for given token- and AccountId, creating new.");
                self.initialize_nonce(acc);
            }
        }
    }

    pub fn initialize_nonce(&mut self, acc: AccountId) {
        debug!("initializing nonce for acc: {:?}", acc);
        self.storage.insert(acc.encode(), NonceHandler::initialize());
    }
}