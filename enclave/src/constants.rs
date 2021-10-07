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

/// Polkadex Defines 1 token as 10^^18 fundamental units
pub const UNIT: u128 = 1000000000000000000;

// Polkadex Module Constants ( this should be updated if order of modules in the runtime is changed
pub static OCEX_MODULE: u8 = 36u8;
pub static OCEX_REGISTER: u8 = 3u8;
pub static OCEX_ADD_PROXY: u8 = 4u8;
pub static OCEX_REMOVE_PROXY: u8 = 5u8;
pub static OCEX_DEPOSIT: u8 = 0u8;
pub static OCEX_RELEASE: u8 = 1u8;
pub static OCEX_WITHDRAW: u8 = 2u8;
pub static OCEX_UPLOAD_CID: u8 = 6u8;

// bump this to be consistent with SubstraTEE-node runtime
pub static RUNTIME_SPEC_VERSION: u32 = 265;
pub static RUNTIME_TRANSACTION_VERSION: u32 = 2;
