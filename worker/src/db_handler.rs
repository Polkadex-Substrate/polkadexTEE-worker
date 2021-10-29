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
// along with this program. If not, see <https://www.gnu.org/licenses/>

use crate::constants::CHUNK_SIZE;
use crate::enclave::api::{enclave_run_db_thread, enclave_send_disk_data};
use crate::enclave_account;
use crate::ipfs_polkadex::Cid;
use crate::polkadex_db::{
    initialize_balances_mirror, initialize_nonce_mirror, initialize_orderbook_mirror,
    load_balances_mirror, load_nonce_mirror, load_orderbook_mirror, PolkadexDBError,
};
use crate::EnclaveBase;
use crate::WsRpcClient;
use codec::Encode;
use frame_support::sp_runtime::app_crypto::sp_core::sr25519;
use log::debug;
use polkadex_sgx_primitives::StorageData;
use sgx_types::sgx_enclave_id_t;
use std::convert::TryFrom;
use std::thread;
use substrate_api_client::Api;

pub struct DBHandler {}

impl DBHandler {
    pub fn initialize_mirrors() {
        initialize_nonce_mirror();
        initialize_balances_mirror();
        initialize_orderbook_mirror();
    }

    pub fn load_balances_from_ipfs<E: EnclaveBase>(
        api: &Api<sr25519::Pair, WsRpcClient>,
        enclave: &E,
    ) -> Result<(), PolkadexDBError> {
        let get_cid: Option<Vec<u8>> = api
            .get_storage_map("PolkadexOcex", "Snapshot", enclave_account(enclave), None)
            .map_err(|_| {
                PolkadexDBError::IpfsError(String::from("Failed to get CID from the chain"))
            })?;

        debug!("CID FROM CHAIN: {:?}", get_cid);

        if let Some(cid) = get_cid {
            let mut balances = load_balances_mirror()?
                .lock()
                .map_err(|_| PolkadexDBError::UnableToLockMutex)?;

            balances.write_data_to_disk(
                crate::ipfs_polkadex::read_from_ipfs(Cid::try_from(cid).map_err(|_| {
                    PolkadexDBError::IpfsError(String::from("Failed to build CID"))
                })?)
                .map_err(|_| {
                    PolkadexDBError::IpfsError(String::from("Failed to read data from ipfs"))
                })?,
            )?
        };
        Ok(())
    }

    pub fn load_from_disk() -> Result<(), PolkadexDBError> {
        let mut balances = load_balances_mirror()?
            .lock()
            .map_err(|_| PolkadexDBError::UnableToLockMutex)?;

        if balances.load_disk_snapshot().is_err() {
            debug!("Balances doesn't have a disk snapshot, proceeding anyway.");
        } else {
            debug!("Balances disk snapshot loaded.");
        }
        let mut nonce = load_nonce_mirror()?
            .lock()
            .map_err(|_| PolkadexDBError::UnableToLockMutex)?;
        if nonce.load_disk_snapshot().is_err() {
            debug!("Nonce doesn't have a disk snapshot, proceeding anyway.");
        } else {
            debug!("Nonce disk snapshot loaded.");
        }
        let mut orderbook = load_orderbook_mirror()?
            .lock()
            .map_err(|_| PolkadexDBError::UnableToLockMutex)?;
        if orderbook.load_disk_snapshot().is_err() {
            debug!("Orderbook doesn't have a disk snapshot, proceeding anyway.");
        } else {
            debug!("Orderbook disk snapshot loaded.");
        }

        debug!(
            "mirrors:\nbalances: {:#?}\nnonce: {:#?}\norderbook: {:#?}",
            *balances, *nonce, *orderbook
        );

        Ok(())
    }

    pub fn initialize(eid: sgx_enclave_id_t) {
        thread::spawn(move || -> Result<(), String> {
            if enclave_run_db_thread(eid).is_err() {
                Err(String::from("Failed to run DB Thread"))
            } else {
                Ok(())
            }
        });
    }

    pub fn send_data_to_enclave(eid: sgx_enclave_id_t) -> Result<(), PolkadexDBError> {
        let balances = load_balances_mirror()?
            .lock()
            .map_err(|_| PolkadexDBError::UnableToLockMutex)?;
        let nonce = load_nonce_mirror()?
            .lock()
            .map_err(|_| PolkadexDBError::UnableToLockMutex)?;
        let orderbook = load_orderbook_mirror()?
            .lock()
            .map_err(|_| PolkadexDBError::UnableToLockMutex)?;

        let balances_data = balances.prepare_for_sending()?;
        let nonce_data = nonce.prepare_for_sending()?;
        let orderbook_data = orderbook.prepare_for_sending()?;

        let (mut balances_chunks, mut nonce_chunks, mut orderbook_chunks) = (
            balances_data.chunks(CHUNK_SIZE),
            nonce_data.chunks(CHUNK_SIZE),
            orderbook_data.chunks(CHUNK_SIZE),
        );
        loop {
            let balances = if let Some(chunk) = balances_chunks.next() {
                chunk.to_vec()
            } else {
                vec![]
            };
            let nonce = if let Some(chunk) = nonce_chunks.next() {
                chunk.to_vec()
            } else {
                vec![]
            };
            let orderbook = if let Some(chunk) = orderbook_chunks.next() {
                chunk.to_vec()
            } else {
                vec![]
            };
            if (balances.clone(), nonce.clone(), orderbook.clone()) == (vec![], vec![], vec![]) {
                break;
            }
            enclave_send_disk_data(
                eid,
                StorageData {
                    balances,
                    nonce,
                    orderbook,
                }
                .encode(),
            )
            .map_err(|_| PolkadexDBError::SendToEnclaveError)?;
        }

        Ok(())
    }
}
