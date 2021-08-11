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

pub mod disk_storage_handler;
pub mod general_db;
pub mod orderbook;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests_orderbook_mirror;

// public exports
pub use orderbook::*;
pub mod nonce;
pub use nonce::*;
pub mod balances;
pub use balances::*;
pub use general_db::*;
pub use disk_storage_handler::DiskStorageHandler;
pub use general_db::*;
pub use orderbook::*;

pub type Result<T> = std::result::Result<T, PolkadexDBError>;

use crate::constants::DISK_SNAPSHOT_INTERVAL;
use std::thread;
use std::time::{Duration, SystemTime};
use log::*;

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
    /// Decode Error
    #[allow(dead_code)] //FIXME: remove as soon _read_disk is actually used
    DecodeError(codec::Error),
}

/// Trait for handling permanante storage
pub trait PermanentStorageHandler {
    /// writes a slice of data into permanent storage of choice
    fn write_to_storage(&mut self, data: &[u8]) -> Result<()>;
    /// reads an vector of data from the permanent storage of choice
    fn read_from_storage(&self) -> Result<Vec<u8>>;
}


// Disk snapshot loop
pub fn start_disk_snapshot_loop() {
    thread::spawn(move || {
        let block_production_interval = Duration::from_millis(DISK_SNAPSHOT_INTERVAL);
	    let mut interval_start = SystemTime::now();
        loop {
            if let Ok(elapsed) = interval_start.elapsed() {
                if elapsed >= block_production_interval {
                    // update interval time
                    interval_start = SystemTime::now();

                    // Take snapshots of all storages
                    take_order_book_snapshot();
                    // TODO: Add the following snapshot:
                    // balance
                    // nonce
                } else {
                    // sleep for the rest of the interval
                    thread::sleep(block_production_interval - elapsed);
                }
            }
        }
    });
}


// Disk snapshot loop
pub fn start_ipfs_snapshot_loop() {
    // TODO
}


// take a disk snapshot of orderbookmirror
fn take_order_book_snapshot() {
    if let Ok(mutex) = crate::polkadex_db::orderbook::load_orderbook() {
        if let Ok(mut orderbook_mirror) = mutex.lock() {
            if let Err(e) = orderbook_mirror.take_disk_snapshot() {
                error!("Could not take an orderbook snaphot due to {:?}", e);
            }
        }
    }
}