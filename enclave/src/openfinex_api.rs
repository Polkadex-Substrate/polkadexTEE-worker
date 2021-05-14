/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/

pub extern crate alloc;
use alloc::{
    borrow::ToOwned,
    format,
    slice::{from_raw_parts, from_raw_parts_mut},
    str,
    string::String,
    vec::Vec,
};

use std::backtrace::{self, PrintFormat};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::slice;

use sgx_types::*;

use log::*;
use rustls::{ClientConfig, ClientSession, ServerConfig, ServerSession, Stream};

use webpki::DNSName;

use crate::aes;
use crate::attestation::{create_ra_report_and_signature, DEV_HOSTNAME};
use crate::cert;
use crate::rsa3072;
use crate::utils::UnwrapOrSgxErrorUnexpected;

use codec::{Decode, Encode};



#[no_mangle]
pub unsafe extern "C" fn run_openfinex_client(
    finex_url: *const u8,
    finex_url_size: usize,
) -> sgx_status_t {

    let url_vec: Vec<u8> = from_raw_parts(finex_url, finex_url_size as usize).to_vec();
    let finex_url = match str::from_utf8(&url_vec) {
        Ok(url) => url,
        Err(e) => {
            error!("Decoding OpenFinex URL failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };
    sgx_status_t::SGX_SUCCESS
}