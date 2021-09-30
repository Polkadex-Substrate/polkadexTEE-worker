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

use codec::{Decode, Encode};
use log::*;
use my_node_runtime::{Header, SignedBlock};
use polkadex_sgx_primitives::PolkadexAccount;
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use sgx_types::*;
use sgx_urts::SgxEnclave;
/// keep this api free from chain-specific types!
use std::io::{Read, Write};
use std::{fs::File, path::PathBuf};
use substratee_enclave_api::{
    enclave_base::EnclaveBase, error::Error as EnclaveApiError, Enclave, EnclaveResult,
};
use substratee_settings::files::{ENCLAVE_FILE, ENCLAVE_TOKEN};

// FIXME: These extern c functions should be moved to the other ffis
extern "C" {
    fn accept_pdex_accounts(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        pdex_accounts: *const u8,
        pdex_accounts_size: usize,
    ) -> sgx_status_t;

    fn run_db_thread(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;

    fn send_disk_data(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        encoded_data: *const u8,
        data_size: usize,
    ) -> sgx_status_t;

}

pub fn enclave_init() -> EnclaveResult<Enclave> {
    const LEN: usize = 1024;
    let mut launch_token = [0; LEN];
    let mut launch_token_updated = 0;

    // Step 1: try to retrieve the launch token saved by last transaction
    //         if there is no token, then create a new one.
    //
    // try to get the token saved in $HOME */
    let mut home_dir = PathBuf::new();
    let use_token = match dirs::home_dir() {
        Some(path) => {
            info!("[+] Home dir is {}", path.display());
            home_dir = path;
            true
        }
        None => {
            error!("[-] Cannot get home dir");
            false
        }
    };
    let token_file = home_dir.join(ENCLAVE_TOKEN);
    if use_token {
        match File::open(&token_file) {
            Err(_) => {
                info!(
                    "[-] Token file {} not found! Will create one.",
                    token_file.as_path().to_str().unwrap()
                );
            }
            Ok(mut f) => {
                info!("[+] Open token file success! ");
                match f.read(&mut launch_token) {
                    Ok(LEN) => {
                        info!("[+] Token file valid!");
                    }
                    _ => info!("[+] Token file invalid, will create new token file"),
                }
            }
        }
    }

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: 1 = debug mode, 0 = not debug mode
    #[cfg(not(feature = "production"))]
    let debug = 1;
    #[cfg(feature = "production")]
    let debug = 0;

    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    let enclave = (SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    ))
    .map_err(EnclaveApiError::Sgx)?;

    // Step 3: save the launch token if it is updated
    if use_token && launch_token_updated != 0 {
        // reopen the file with write capability
        match File::create(&token_file) {
            Ok(mut f) => match f.write_all(&launch_token) {
                Ok(()) => info!("[+] Saved updated launch token!"),
                Err(_) => error!("[-] Failed to save updated launch token!"),
            },
            Err(_) => {
                warn!("[-] Failed to save updated enclave token, but doesn't matter");
            }
        }
    }

    // create an enclave API and initialize it
    let enclave_api = Enclave::new(enclave);
    enclave_api.init()?;

    Ok(enclave_api)
}

pub fn enclave_accept_pdex_accounts(
    eid: sgx_enclave_id_t,
    pdex_accounts: Vec<PolkadexAccount>,
) -> SgxResult<()> {
    let mut status = sgx_status_t::SGX_SUCCESS;

    let result = unsafe {
        accept_pdex_accounts(
            eid,
            &mut status,
            pdex_accounts.encode().as_ptr(),
            pdex_accounts.encode().len(),
        )
    };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }

    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }
    Ok(())
}

pub fn enclave_send_disk_data(eid: sgx_enclave_id_t, data: Vec<u8>) -> SgxResult<()> {
    let mut status = sgx_status_t::SGX_SUCCESS;

    let result = unsafe { send_disk_data(eid, &mut status, data.as_ptr(), data.len()) };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }
    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }
    Ok(())
}

pub fn enclave_run_db_thread(eid: sgx_enclave_id_t) -> SgxResult<()> {
    let mut status = sgx_status_t::SGX_SUCCESS;

    let result = unsafe { run_db_thread(eid, &mut status) };

    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }

    if result != sgx_status_t::SGX_SUCCESS {
        return Err(result);
    }

    Ok(())
}
