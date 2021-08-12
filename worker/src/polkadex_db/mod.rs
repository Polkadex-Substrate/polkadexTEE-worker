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

pub mod balances;
pub mod disk_storage_handler;
pub mod ipfs_storage_handler;
pub mod general_db;
#[cfg(test)]
pub mod mock;
pub mod nonce;
pub mod orderbook;
#[cfg(test)]
pub mod tests_orderbook_mirror;

// public exports
pub use balances::*;
pub use disk_storage_handler::DiskStorageHandler;
pub use ipfs_storage_handler::IpfsStorageHandler;
pub use general_db::*;
pub use nonce::*;
pub use orderbook::*;

pub type Result<T> = std::result::Result<T, PolkadexDBError>;

use crate::constants::{SNAPSHOT_INTERVAL, ORDERBOOK_DISK_STORAGE_FILENAME, NONCE_DISK_STORAGE_FILENAME, BALANCE_DISK_STORAGE_FILENAME};
use log::*;
use std::thread;
use std::time::{Duration, SystemTime};
use std::path::PathBuf;


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
pub fn start_snapshot_loop() {
    thread::spawn(move || {
        println!("Successfully started disk snapshot loop");
        let snapshot_interval = Duration::from_millis(SNAPSHOT_INTERVAL);
        let mut interval_start = SystemTime::now();
        loop {
            if let Ok(elapsed) = interval_start.elapsed() {
                if elapsed >= snapshot_interval {
                    // update interval time
                    interval_start = SystemTime::now();

                    // Take snapshots of all storages
                    take_orderbook_snapshot();
                    take_balance_snapshot();
                    take_nonce_snapshot();
                } else {
                    // sleep for the rest of the interval
                    thread::sleep(snapshot_interval - elapsed);
                }
            }
        }
    });
}

// take a disk snapshot of orderbookmirror
fn take_orderbook_snapshot() {
    if let Ok(mutex) = crate::polkadex_db::orderbook::load_orderbook_mirror() {
        if let Ok(mut orderbook_mirror) = mutex.lock() {
            if let Err(e) = orderbook_mirror.take_disk_snapshot() {
                error!("Could not take an orderbook mirror snaphot due to {:?}", e);
            }
        }
    }
}

// take a disk snapshot of balancemirror
fn take_balance_snapshot() {
    if let Ok(mutex) = crate::polkadex_db::balances::load_balances_mirror() {
        if let Ok(mut balance_mirror) = mutex.lock() {
            if let Err(e) = balance_mirror.take_disk_snapshot() {
                error!("Could not take an balnace mirror snaphot due to {:?}", e);
            }
        }
    }
}

// take a disk snapshot of balancemirror
fn take_nonce_snapshot() {
    if let Ok(mutex) = crate::polkadex_db::nonce::load_nonce_mirror() {
        if let Ok(mut nonce_mirror) = mutex.lock() {
            if let Err(e) = nonce_mirror.take_disk_snapshot() {
                error!("Could not take a nonce mirror snaphot due to {:?}", e);
            }
        }
    }
}
