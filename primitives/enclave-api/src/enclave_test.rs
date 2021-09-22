/*
    Copyright 2019 Supercomputing Systems AG
    Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

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

use crate::{error::Error, Enclave, EnclaveResult};
use frame_support::ensure;
use log::*;
use sgx_types::sgx_status_t;
use substratee_enclave_api_ffi as ffi;

pub trait EnclaveTest: Send + Sync + 'static {
    fn test_main_entrance(&self) -> EnclaveResult<()>;
}

impl EnclaveTest for Enclave {
    fn test_main_entrance(&self) -> EnclaveResult<()> {
        let mut retval = sgx_status_t::SGX_SUCCESS;

        let result = unsafe { ffi::test_main_entrance(self.eid, &mut retval) };

        ensure!(result == sgx_status_t::SGX_SUCCESS, Error::Sgx(result));
        ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

        debug!("[+] successfully executed enclave test main");

        Ok(())
    }
}
