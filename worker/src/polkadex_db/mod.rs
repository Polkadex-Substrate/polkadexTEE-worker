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
pub mod general_db;
pub mod ipfs_storage_handler;
#[cfg(test)]
pub mod mock;
pub mod nonce;
pub mod orderbook;
#[cfg(test)]
pub mod tests_orderbook_mirror;

// public exports
pub use balances::*;
pub use disk_storage_handler::DiskStorageHandler;
pub use general_db::*;
pub use ipfs_storage_handler::IpfsStorageHandler;
pub use nonce::*;
pub use orderbook::*;

pub type Result<T> = std::result::Result<T, PolkadexDBError>;

use crate::constants::SNAPSHOT_INTERVAL;
use crate::enclave::api::enclave_send_cid;
use codec::{Decode, Encode};
use log::*;
use my_node_runtime::UncheckedExtrinsic;
use sgx_types::sgx_enclave_id_t;
use sp_core::{sr25519, Pair};
use std::thread;
use std::time::{Duration, SystemTime};
use substrate_api_client::{Api, UncheckedExtrinsicV4, XtStatus};

#[derive(Debug)]
/// Polkadex DB Error
pub enum PolkadexDBError {
    /// Failed to load pointer
    UnableToLoadPointer,
    /// Failed to lock mutex
    UnableToLockMutex,
    /// Failed to deserialize value
    UnableToDeseralizeValue,
    /// Failed to find key in the DB
    KeyNotFound,
    /// File system interaction error
    FsError(std::io::Error),
    /// Decode Error
    DecodeError(codec::Error),
    /// Failed to send data to enclave
    SendToEnclaveError,
    // Could not send IPFS snapshot
    IpfsError(String),
}

/// Trait for handling permanante storage
pub trait PermanentStorageHandler {
    /// writes a slice of data into permanent storage of choice
    fn write_to_storage(&mut self, data: &[u8]) -> Result<()>;
    /// reads an vector of data from the permanent storage of choice
    fn read_from_storage(&self) -> Result<Vec<u8>>;
}

// Disk snapshot loop
pub fn start_snapshot_loop(
    api: Api<sr25519::Pair>,
    eid: sgx_enclave_id_t,
    genesis_hash: Vec<u8>,
    w_url: Vec<u8>,
    nonce: u32,
) {
    thread::spawn(move || {
        println!("Successfully started snapshot loop");
        let snapshot_interval = Duration::from_millis(SNAPSHOT_INTERVAL);
        let mut interval_start = SystemTime::now();
        loop {
            if let Ok(elapsed) = interval_start.elapsed() {
                if elapsed >= snapshot_interval {
                    // update interval time
                    interval_start = SystemTime::now();

                    // Take snapshots of all storages
                    if let Err(e) = take_orderbook_snapshot() {
                        error!("Could not take orderbook snapshot: {:?}", e);
                    }
                    if let Err(e) = take_balance_snapshot(
                        api.clone(),
                        eid,
                        genesis_hash.clone(),
                        w_url.clone(),
                        nonce,
                    ) {
                        error!("Could not take balance snapshot: {:?}", e);
                    };
                    if let Err(e) = take_nonce_snapshot() {
                        error!("Could not take nonce snapshot: {:?}", e);
                    };
                } else {
                    // sleep for the rest of the interval
                    thread::sleep(snapshot_interval - elapsed);
                }
            }
        }
    });
}

// take a snapshot of orderbookmirror
fn take_orderbook_snapshot() -> Result<()> {
    let mutex = crate::polkadex_db::orderbook::load_orderbook_mirror()?;
    let mut orderbook_mirror = mutex
        .lock()
        .map_err(|_| PolkadexDBError::UnableToLoadPointer)?;
    orderbook_mirror.take_disk_snapshot()?;
    Ok(())
}

// take a snapshot of balances mirror
fn take_balance_snapshot(
    api: Api<sr25519::Pair>,
    eid: sgx_enclave_id_t,
    genesis_hash: Vec<u8>,
    w_url: Vec<u8>,
    nonce: u32,
) -> Result<()> {
    let mutex = crate::polkadex_db::balances::load_balances_mirror()?;
    let mut balance_mirror = mutex
        .lock()
        .map_err(|_| PolkadexDBError::UnableToLoadPointer)?;
    let data = balance_mirror.take_disk_snapshot()?;
    let mut ipfs_handler = IpfsStorageHandler::default();
    let cid = ipfs_handler.snapshot_to_ipfs(data)?;
    error!("Retrived cid {:?} for balance snapshot", cid);

    error!("cid encoded: {:?}", cid.to_bytes());

    let (uxt, size) = enclave_send_cid(eid, genesis_hash, nonce, cid.to_bytes(), w_url).unwrap();

    error!(
        "data: {:?}, size: {}",
        uxt.as_slice().split_at(size as usize).0,
        size
    );

    let ue = UncheckedExtrinsic::decode(&mut uxt.as_slice().split_at(size as usize).0).unwrap();

    let mut _xthex = hex::encode(ue.encode());
    _xthex.insert_str(0, "0x");
    error!("ext: {:?}", _xthex);

    // send the extrinsic and wait for confirmation
    error!(
        "send extrinsic: {:?}",
        api.send_extrinsic(_xthex, XtStatus::Finalized)
    );

    Ok(())
}

// take a snapshot of nonce mirror
fn take_nonce_snapshot() -> Result<()> {
    let mutex = crate::polkadex_db::nonce::load_nonce_mirror()?;
    let mut nonce_mirror = mutex
        .lock()
        .map_err(|_| PolkadexDBError::UnableToLoadPointer)?;
    nonce_mirror.take_disk_snapshot()?;
    Ok(())
}
