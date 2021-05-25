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

use crate::attestation;
use crate::polkadex_balance_storage::lock_storage_and_get_balances;
use crate::rpc::rpc_call::{RpcCall, RpcMethodImpl};
use crate::rpc::rpc_call_encoder::JsonRpcCallEncoder;
use crate::rpc::rpc_info::{RpcCallStatus, RpcInfo};
use base58::ToBase58;
use codec::{Decode, Encode};
use jsonrpc_core::Result as RpcResult;
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use sgx_types::sgx_measurement_t;
use substratee_stf::{
    Getter, ShardIdentifier, TrustedCall, TrustedCallSigned, TrustedGetter, TrustedOperation,
};
use substratee_worker_primitives::DirectRequestStatus;

/// Get a list of all RPC calls - can be used to insert into the IO handler
pub fn get_all_rpc_calls() -> Vec<RpcCall<JsonRpcCallEncoder, RpcMethodImpl>> {
    vec![
        RpcCall::new("place_order", &place_order, JsonRpcCallEncoder {}),
        RpcCall::new("cancel_order", &cancel_order, JsonRpcCallEncoder {}),
        RpcCall::new("withdraw", &withdraw, JsonRpcCallEncoder {}),
        RpcCall::new("get_balance", &get_balance, JsonRpcCallEncoder {}),
    ]
}

fn place_order(request: DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)> {
    debug!("entering place_order RPC");

    // TODO the functionality of verifying the request and extracting the parameters is duplicated
    // in each function. Generalize it and share it among all calls
    let verified_trusted_operation = get_verified_trusted_operation(request);
    if let Err(s) = verified_trusted_operation {
        return Ok((RpcInfo::from(s), false, DirectRequestStatus::Error));
    }

    let place_order_call_args = match verified_trusted_operation.unwrap() {
        TrustedOperation::direct_call(tcs) => match tcs.call {
            TrustedCall::place_order(a, o, p) => Ok((a, o, p)),
            _ => Err(RpcCallStatus::operation_type_mismatch),
        },
        _ => Err(RpcCallStatus::operation_type_mismatch),
    };

    if let Err(e) = place_order_call_args {
        return Ok((RpcInfo::from(e), false, DirectRequestStatus::Error));
    }

    // TODO call implementation here

    Ok((
        RpcInfo::from(RpcCallStatus::operation_success),
        false,
        DirectRequestStatus::Ok,
    ))
}

fn cancel_order(request: DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)> {
    debug!("entering cancel_order RPC");

    let verified_trusted_operation = get_verified_trusted_operation(request);
    if let Err(s) = verified_trusted_operation {
        return Ok((RpcInfo::from(s), false, DirectRequestStatus::Error));
    }

    let cancel_order_call_args = match verified_trusted_operation.unwrap() {
        TrustedOperation::direct_call(tcs) => match tcs.call {
            TrustedCall::cancel_order(a, o, p) => Ok((a, o, p)),
            _ => Err(RpcCallStatus::operation_type_mismatch),
        },
        _ => Err(RpcCallStatus::operation_type_mismatch),
    };

    if let Err(e) = cancel_order_call_args {
        return Ok((RpcInfo::from(e), false, DirectRequestStatus::Error));
    }

    // TODO call implementation here

    Ok((
        RpcInfo::from(RpcCallStatus::operation_success),
        false,
        DirectRequestStatus::Ok,
    ))
}

fn withdraw(request: DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)> {
    debug!("entering withdraw RPC");

    let verified_trusted_operation = get_verified_trusted_operation(request);
    if let Err(s) = verified_trusted_operation {
        return Ok((RpcInfo::from(s), false, DirectRequestStatus::Error));
    }

    let withdraw_call_args = match verified_trusted_operation.unwrap() {
        TrustedOperation::direct_call(tcs) => match tcs.call {
            TrustedCall::withdraw(a, c, b, p) => Ok((a, c, b, p)),
            _ => Err(RpcCallStatus::operation_type_mismatch),
        },
        _ => Err(RpcCallStatus::operation_type_mismatch),
    };

    if let Err(e) = withdraw_call_args {
        return Ok((RpcInfo::from(e), false, DirectRequestStatus::Error));
    }

    // TODO call implementation here

    Ok((
        RpcInfo::from(RpcCallStatus::operation_success),
        false,
        DirectRequestStatus::Ok,
    ))
}

fn get_balance(request: DirectRequest) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)> {
    debug!("entering get_balance RPC");

    let verified_trusted_operation = get_verified_trusted_operation(request);
    if let Err(s) = verified_trusted_operation {
        return Ok((RpcInfo::from(s), false, DirectRequestStatus::Error));
    }

    let get_balance_call_args = match verified_trusted_operation.unwrap() {
        TrustedOperation::get(getter) => match getter {
            Getter::trusted(tgs) => match tgs.getter {
                TrustedGetter::get_balance(a, c, p) => Ok((p.unwrap_or(a), c)),
                _ => Err(RpcCallStatus::operation_type_mismatch),
            },
            _ => Err(RpcCallStatus::operation_type_mismatch),
        },
        _ => Err(RpcCallStatus::operation_type_mismatch),
    };

    if let Err(e) = get_balance_call_args {
        return Ok((RpcInfo::from(e), false, DirectRequestStatus::Error));
    }

    //let main_account = get_balance_call_args.0;
    //let asset_id = get_balance_call_args.1;
    //let balances_result = lock_storage_and_get_balances(main_account asset_id);

    Ok((
        RpcInfo::from(RpcCallStatus::operation_success),
        false,
        DirectRequestStatus::Ok,
    ))
}

fn get_verified_trusted_operation(
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
        Err(e) => Err(RpcCallStatus::decoding_failure),
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
