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

pub extern crate alloc;
use alloc::{string::String, string::ToString};

use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::{RpcCallStatus, RpcInfo};
use crate::rpc::trusted_operation_verifier::get_verified_trusted_operation;
use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::*;
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use substratee_stf::{TrustedCall, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcPlaceOrder {}

impl RpcPlaceOrder {
    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> RpcResult<(RpcInfo, bool, DirectRequestStatus)> {
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
}

impl RpcCall for RpcPlaceOrder {
    fn name() -> String {
        "place_order".to_string()
    }
}

impl RpcMethodSync for RpcPlaceOrder {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}
