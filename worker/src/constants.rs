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

pub static ENCRYPTED_STATE_FILE: &str = "state.bin";
pub static SHARDS_PATH: &str = "./shards";
pub static ENCLAVE_TOKEN: &str = "../bin/enclave.token";
pub static ENCLAVE_FILE: &str = "../bin/enclave.signed.so";
pub static SHIELDING_KEY_FILE: &str = "enclave-shielding-pubkey.json";
pub static SIGNING_KEY_FILE: &str = "enclave-signing-pubkey.bin";
//pub static ORDERBOOK_LAST_COUNTER: &str = "LAST_ORDER_COUNTER";

#[cfg(feature = "production")]
pub static RA_SPID_FILE: &str = "../bin/spid_production.txt";
#[cfg(feature = "production")]
pub static RA_API_KEY_FILE: &str = "../bin/key_production.txt";

#[cfg(not(feature = "production"))]
pub static RA_SPID_FILE: &str = "../bin/spid.txt";
#[cfg(not(feature = "production"))]
pub static RA_API_KEY_FILE: &str = "../bin/key.txt";



// the maximum size of any extrinsic that the enclave will ever generate in B
pub static EXTRINSIC_MAX_SIZE: usize = 4196;
// the maximum size of a value that will be queried from the state in B
pub static STATE_VALUE_MAX_SIZE: usize = 1024;
// Iterator for RocksDB returns these many elements in a single yield
pub static ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT: usize = 1000;
pub static ORDERBOOK_DISK_STORAGE_FILENAME: &str = "orderbook.bin";
