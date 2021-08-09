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

use crate::enclave::api::enclave_run_db_thread;
use crate::polkadex_db::{
    initialize_balances_mirror, initialize_nonce_mirror, initialize_orderbook_mirror,
};
use sgx_types::sgx_enclave_id_t;
use std::thread;

pub struct DBHandler {}

impl DBHandler {
    fn initialize_mirrors() {
        initialize_nonce_mirror();
        initialize_balances_mirror();
        initialize_orderbook_mirror();
    }

    pub fn initialize(eid: sgx_enclave_id_t) {
        DBHandler::initialize_mirrors();
        thread::spawn(move || -> Result<(), String> {
            if enclave_run_db_thread(eid).is_err() {
                return Err(String::from("Failed to run DB Thread"));
            }
            Ok(())
        });
    }
}
