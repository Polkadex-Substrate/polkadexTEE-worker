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

use codec::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Copy)]
pub struct NonceHandler {
    pub nonce: Option<u32>
}

impl NonceHandler {
    pub fn create() -> Self {
        Self {
            nonce: None
        }
    }

    pub fn initialize() -> Self {
        Self {
            nonce: Some(0u32)
        }
    }

    pub fn increment(&mut self) {
        if let Some(inner_nonce) = self.nonce {
            self.nonce = Some(inner_nonce + 1u32)
        }
        else {
            self.nonce = Some(0u32)
        }
    }

    pub fn update(&mut self, nonce: u32) {
        self.nonce = Some(nonce);
    }
}