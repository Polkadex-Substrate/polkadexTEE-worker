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

pub mod general_db;
pub mod disc_storage_handler;
pub use general_db::*;
pub mod orderbook;
pub use orderbook::*;
pub mod nonce;
pub use nonce::*;
pub mod balances;
pub use balances::*;

pub type Result<T> = std::result::Result<T, PolkadexDBError>;

#[derive(Debug)]
/// Polkadex DB Error
pub enum PolkadexDBError {
    /// Failed to load pointer
    UnableToLoadPointer,
    /// Failed to deserialize value
    UnableToDeseralizeValue,
    /// Failed to find key in the DB
    _KeyNotFound,
    /// Failed to decode
    _DecodeError,
    /// File system interaction error
    FsError(std::io::Error),
}

/// Trait for handling permanante storage
pub trait PermanentStorageHandler {
    /// writes a slice of data into permanent storage of choice
    fn write_to_storage(&self, data: &[u8]) -> Result<()>;
    /// reads an vector of data from the permanent storage of choice
    fn read_from_storage(&self) -> Result<Vec<u8>>;
}
