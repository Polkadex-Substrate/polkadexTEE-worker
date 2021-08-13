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

use crate::enclave::api::{enclave_run_db_thread, enclave_send_disk_data};
use crate::polkadex_db::{
    initialize_balances_mirror, initialize_nonce_mirror, initialize_orderbook_mirror,
    load_balances_mirror, load_nonce_mirror, load_orderbook_mirror,
};
use codec::Encode;
use polkadex_sgx_primitives::{OrderbookData, StorageData};
use sgx_types::sgx_enclave_id_t;
use std::thread;

pub struct DBHandler {}

impl DBHandler {
    fn initialize_mirrors() {
        initialize_nonce_mirror();
        initialize_balances_mirror();
        initialize_orderbook_mirror();
    }

    pub fn load_from_disk() -> Result<(), String> {
        DBHandler::initialize_mirrors();
        let mut balances = load_balances_mirror()
            .map_err(|_| String::from("Failed to load balances mirror"))?
            .lock()
            .map_err(|_| String::from("Failed to lock mutex"))?;
        balances
            .load_disk_snapshot()
            .map_err(|_| String::from("Failed to load balances snapshot"))?;
        let mut nonce = load_nonce_mirror()
            .map_err(|_| String::from("Failed to load nonce mirror"))?
            .lock()
            .map_err(|_| String::from("Failed to lock mutex"))?;
        nonce
            .load_disk_snapshot()
            .map_err(|_| String::from("Failed to load nonce snapshot"))?;
        let mut orderbook = load_orderbook_mirror()
            .map_err(|_| String::from("Failed to load orderbook mirror"))?
            .lock()
            .map_err(|_| String::from("Failed to lock mutex"))?;
        orderbook
            .load_disk_snapshot()
            .map_err(|_| String::from("Failed to load orderbook snapshot"))?;

        log::debug!(
            "mirrors:\nbalances: {:#?}\nnonce: {:#?}\norderbook: {:#?}",
            *balances,
            *nonce,
            *orderbook
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

    pub fn send_data_to_enclave(eid: sgx_enclave_id_t) -> Result<(), String> {
        let balances = load_balances_mirror()
            .map_err(|_| String::from("Failed to load balances mirror"))?
            .lock()
            .map_err(|_| String::from("Failed to lock mutex"))?;
        let nonce = load_nonce_mirror()
            .map_err(|_| String::from("Failed to load nonce mirror"))?
            .lock()
            .map_err(|_| String::from("Failed to lock mutex"))?;
        let orderbook = load_orderbook_mirror()
            .map_err(|_| String::from("Failed to load orderbook mirror"))?
            .lock()
            .map_err(|_| String::from("Failed to lock mutex"))?;
        enclave_send_disk_data(
            eid,
            StorageData {
                balances: balances.prepare(),
                nonce: nonce.prepare(),
                orderbook: orderbook
                    .read_all()
                    .unwrap()
                    .into_iter()
                    .map(|signed_order| OrderbookData { signed_order })
                    .collect(),
            }
            .encode(),
        )
        .map_err(|_| String::from("Failed to send data to enclave"))?;

        Ok(())
    }
}
