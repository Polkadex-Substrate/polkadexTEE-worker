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
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use substratee_stf::{TrustedCall, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcCancelOrder {}

impl RpcCancelOrder {
    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> Result<(RpcInfo, bool, DirectRequestStatus), String> {
        debug!("entering cancel_order RPC");

        let verified_trusted_operation = get_verified_trusted_operation(request)?;

        let cancel_order_call_args = match verified_trusted_operation {
            TrustedOperation::direct_call(tcs) => match tcs.call {
                TrustedCall::cancel_order(a, o, p) => Ok((a, o, p)),
                _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
            },
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        // TODO call implementation here

        Ok((
            RpcInfo::from(RpcCallStatus::operation_success),
            false,
            DirectRequestStatus::Ok,
        ))
    }
}

impl RpcCall for RpcCancelOrder {
    fn name() -> String {
        "cancel_order".to_string()
    }
}

impl RpcMethodSync for RpcCancelOrder {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}
