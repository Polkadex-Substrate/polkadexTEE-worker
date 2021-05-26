// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü.
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

use crate::attestation;
use crate::rpc::rpc_info::RpcCallStatus;
use base58::ToBase58;
use codec::Decode;
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use polkadex_sgx_primitives::ShardIdentifier;
use sgx_types::sgx_measurement_t;
use substratee_stf::{Getter, TrustedCallSigned, TrustedOperation};

pub fn get_verified_trusted_operation(
    request: DirectRequest,
) -> Result<TrustedOperation, RpcCallStatus> {
    // decode call
    let shard_id = request.shard;

    let trusted_operation = match decode_request(request) {
        Ok(decoded_result) => decoded_result,
        Err(e) => return Err(e),
    };

    match verify_signature(&trusted_operation, &shard_id) {
        Ok(()) => {
            debug!("successfully verified signature")
        }
        Err(e) => return Err(e),
    }

    Ok(trusted_operation)
}

fn decode_request(request: DirectRequest) -> Result<TrustedOperation, RpcCallStatus> {
    debug!("decode Request -> TrustedOperation");
    match TrustedOperation::decode(&mut request.encoded_text.as_slice()) {
        Ok(trusted_operation) => Ok(trusted_operation),
        Err(_) => Err(RpcCallStatus::decoding_failure),
    }
}

fn verify_signature(
    top: &TrustedOperation,
    shard_id: &ShardIdentifier,
) -> Result<(), RpcCallStatus> {
    debug!("verify signature of TrustedOperation");
    debug!("query mrenclave of self");
    let mrenclave = match attestation::get_mrenclave_of_self() {
        Ok(m) => m,
        Err(_) => return Err(RpcCallStatus::mrenclave_failure),
    };

    debug!("MRENCLAVE of self is {}", mrenclave.m.to_base58());

    match top {
        TrustedOperation::direct_call(tcs) => {
            verify_signature_of_signed_call(tcs, &mrenclave, shard_id)
        }
        TrustedOperation::indirect_call(tcs) => {
            verify_signature_of_signed_call(tcs, &mrenclave, shard_id)
        }
        TrustedOperation::get(getter) => {
            match getter {
                Getter::public(_) => Ok(()), // no need to verify signature on public getter
                Getter::trusted(tgs) => {
                    if let true = tgs.verify_signature() {
                        return Ok(());
                    }
                    return Err(RpcCallStatus::signature_verification_failure);
                }
            }
        }
    }
}

fn verify_signature_of_signed_call(
    trusted_call: &TrustedCallSigned,
    mrenclave: &sgx_measurement_t,
    shard_id: &ShardIdentifier,
) -> Result<(), RpcCallStatus> {
    if trusted_call.verify_signature(&mrenclave.m, shard_id) {
        return Ok(());
    }

    Err(RpcCallStatus::signature_verification_failure)
}
