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

    pub fn load_from_disk() {
        DBHandler::initialize_mirrors();
        let mut balances = load_balances_mirror().unwrap().lock().unwrap();
        balances.load_disk_snapshot().unwrap_or(());
        let mut nonce = load_nonce_mirror().unwrap().lock().unwrap();
        nonce.load_disk_snapshot().unwrap_or(());
        let mut orderbook = load_orderbook_mirror().unwrap().lock().unwrap();
        orderbook.load_disk_snapshot().unwrap_or(());

        log::debug!(
            "mirrors:\nbalances: {:#?}\nnonce: {:#?}\norderbook: {:#?}",
            *balances,
            *nonce,
            *orderbook
        )
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

    pub fn send_data_to_enclave(eid: sgx_enclave_id_t) {
        let balances = load_balances_mirror().unwrap().lock().unwrap();
        let nonce = load_nonce_mirror().unwrap().lock().unwrap();
        let orderbook = load_orderbook_mirror().unwrap().lock().unwrap();
        log::error!(
            "sent disk data: {:#?}",
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
                        .collect()
                }
                .encode()
            )
        );
    }
}
