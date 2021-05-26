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
use alloc::{str, string::String, string::ToString, vec::Vec};
use codec::Error as CodecError;
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

// impl Encode for RpcInfo {
//     fn encode(&self) -> Vec<u8> {
//         self.status.encode()
//     }
// }
//
// impl Decode for RpcInfo {
//     fn decode<T>(input: &mut T) -> Result<Self, CodecError> {
//         return match String::decode(input) {
//             Ok(s) => Ok(RpcInfo::from(s)),
//             Err(e) => Err(e),
//         };
//     }
// }
