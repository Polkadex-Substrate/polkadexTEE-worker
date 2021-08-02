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
use alloc::{string::String, string::ToString};
use codec::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum RpcCallStatus {
    operation_type_mismatch,
    signature_verification_failure,
    decoding_failure,
    mrenclave_failure,
    operation_success,
}

impl ToString for RpcCallStatus {
    fn to_string(&self) -> String {
        match self {
            RpcCallStatus::operation_type_mismatch => String::from("Operation types do not match"),
            RpcCallStatus::signature_verification_failure => {
                String::from("Failed to verify signature")
            }
            RpcCallStatus::decoding_failure => String::from("Failed to decode"),
            RpcCallStatus::mrenclave_failure => String::from("Failed to get MRENCLAVE"),
            RpcCallStatus::operation_success => String::from("Operation was successful"),
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct RpcInfo {
    pub status: RpcCallStatus,
    // originally we wanted to have String, but String in the enclave does not
    // implement the Decode/Encode trait properly, so even wrapping it with this struct
    // was not successful. The workaround in the meantime is to use just enums
}

impl RpcInfo {
    pub fn from(s: RpcCallStatus) -> Self {
        RpcInfo { status: s }
    }
}
