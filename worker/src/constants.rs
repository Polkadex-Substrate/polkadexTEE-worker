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

// FIXME: deprecated file location. Should probably be moved to polkadex_primitives or something like that

// Iterator for orderbook mirror returns these many elements in a single yield
pub static ORDERBOOK_MIRROR_ITERATOR_YIELD_LIMIT: usize = 1000;

// polkadex DB file names
pub static DEFAULT_STORAGE_PATH: &str = "polkadex_storage";
pub static ORDERBOOK_DISK_STORAGE_FILENAME: &str = "orderbook.bin";
pub static NONCE_DISK_STORAGE_FILENAME: &str = "nonce.bin";
pub static BALANCE_DISK_STORAGE_FILENAME: &str = "balance.bin";

// IPFS gateway
pub static IPFS_HOST: &str = "localhost";
pub static IPFS_PORT: u16 = 5001;

// Interval of disk snapshot
pub static SNAPSHOT_INTERVAL: u64 = 1000; // ms

// Chunk size for sending data to enclave
pub static CHUNK_SIZE: usize = 1000;
