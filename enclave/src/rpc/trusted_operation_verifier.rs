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

pub extern crate alloc;
use alloc::{string::String, string::ToString};

use crate::attestation;
use crate::rpc::rpc_info::RpcCallStatus;
use base58::ToBase58;
use codec::Decode;
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use polkadex_sgx_primitives::ShardIdentifier;
use sgx_types::sgx_measurement_t;
use substratee_stf::{Getter, TrustedCallSigned, TrustedOperation};

pub trait TrustedOperationExtractor: Send + Sync {
    fn get_verified_trusted_operation(
        &self,
        request: DirectRequest,
    ) -> Result<TrustedOperation, String>;
}

pub struct TrustedOperationVerifier {}

impl TrustedOperationExtractor for TrustedOperationVerifier {
    fn get_verified_trusted_operation(
        &self,
        request: DirectRequest,
    ) -> Result<TrustedOperation, String> {
        get_verified_trusted_operation(request)
    }
}

pub fn get_verified_trusted_operation(request: DirectRequest) -> Result<TrustedOperation, String> {
    // decode call
    let shard_id = request.shard;

    let trusted_operation = match decode_request(request) {
        Ok(decoded_result) => decoded_result,
        Err(e) => return Err(e.to_string()),
    };

    match verify_signature(&trusted_operation, &shard_id) {
        Ok(()) => {
            debug!("successfully verified signature")
        }
        Err(e) => return Err(e.to_string()),
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
        //FIXME: This is not returning the correct value
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
    _mrenclave: &sgx_measurement_t,
    shard_id: &ShardIdentifier,
) -> Result<(), RpcCallStatus> {
    let m = [0u8; 32];
    if trusted_call.verify_signature(
        //&mrenclave.m
        &m, &shard_id,
    ) {
        return Ok(());
    }

    Err(RpcCallStatus::signature_verification_failure)
}

pub mod tests {

    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_request, sign_trusted_call,
    };
    use codec::Encode;
    use polkadex_sgx_primitives::{AccountId, AssetId};
    use sp_core::{ed25519 as ed25519_core, Pair, H256};
    use substratee_stf::TrustedCall;
    use crate::ShardIdentifier;

    pub fn given_valid_operation_in_request_then_decode_succeeds() {
        let input_trusted_operation = create_trusted_operation();
        let request = DirectRequest {
            encoded_text: input_trusted_operation.encode(),
            shard: ShardIdentifier::default(),
        };

        let decoded_operation =
            get_verified_trusted_operation(request).expect("Failed to verify operation.");

        match decoded_operation {
            TrustedOperation::direct_call(tcs) => match tcs.call {
                TrustedCall::withdraw(_, asset_id, amount, proxy) => {
                    assert_eq!(asset_id, AssetId::POLKADEX);
                    assert_eq!(amount, 14875210);
                    assert!(proxy.is_none());
                }
                _ => assert!(false, "got unexpected TrustedCall back from decoding"),
            },
            _ => assert!(false, "got unexpected TrustedOperation back from decoding"),
        }
    }

    pub fn given_nonsense_text_in_request_then_decode_fails() {
        let invalid_request = create_dummy_request();

        let top_result = decode_request(invalid_request);

        assert!(top_result.is_err());
    }

    pub fn given_valid_operation_with_invalid_signature_then_return_error() {
        let invalid_top = create_trusted_operation_with_incorrect_signature();
        let request = DirectRequest {
            encoded_text: invalid_top.encode(),
            shard: H256::from([1u8; 32]),
        };

        let top_result = get_verified_trusted_operation(request);

        assert!(top_result.is_err());

        match top_result {
            Ok(_) => assert!(false, "did not expect Ok result"),
            Err(e) => {
                assert_eq!(e, RpcCallStatus::signature_verification_failure.to_string())
            }
        }
    }

    fn create_trusted_operation() -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let trusted_call =
            TrustedCall::withdraw(account_id.clone(), AssetId::POLKADEX, 14875210, None);
        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair, 0u32);

        TrustedOperation::direct_call(trusted_call_signed)
    }

    fn create_trusted_operation_with_incorrect_signature() -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let malicious_signer = ed25519_core::Pair::from_seed(b"19857777701234567890123456789012");

        let trusted_call =
            TrustedCall::withdraw(account_id.clone(), AssetId::POLKADEX, 14875210, None);

        let trusted_call_signed = sign_trusted_call(trusted_call, malicious_signer, 0u32);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
