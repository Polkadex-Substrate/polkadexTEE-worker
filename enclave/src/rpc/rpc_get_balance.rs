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

use crate::polkadex_balance_storage::{lock_storage_and_get_balances, Balances};
use crate::polkadex_gateway::authenticate_user;
use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::{RpcCallStatus, RpcInfo};
use crate::rpc::trusted_operation_verifier::get_verified_trusted_operation;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use substratee_stf::{Getter, TrustedGetter, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcGetBalance {}

impl RpcGetBalance {
    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> Result<(Balances, bool, DirectRequestStatus), String> {
        debug!("entering get_balance RPC");

        let verified_trusted_operation = get_verified_trusted_operation(request)?;

        let trusted_getter_signed = match verified_trusted_operation {
            TrustedOperation::get(getter) => match getter {
                Getter::trusted(tgs) => Ok(tgs),
                _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
            },
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let main_account = trusted_getter_signed.getter.main_account().clone();
        let proxy_account = trusted_getter_signed.getter.proxy_account().clone();

        let _authorization_result = match authenticate_user(main_account.clone(), proxy_account) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Authorization error: {}", e)),
        }?;

        let asset_id = match trusted_getter_signed.getter {
            TrustedGetter::get_balance(_, c, _) => Ok(c),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let balances = match lock_storage_and_get_balances(main_account.clone(), asset_id) {
            Ok(b) => Ok(b),
            Err(e) => Err(String::from(e.as_str())),
        }?;

        Ok((balances, false, DirectRequestStatus::Ok))
    }
}

impl RpcCall for RpcGetBalance {
    fn name() -> String {
        "get_balance".to_string()
    }
}

impl RpcMethodSync for RpcGetBalance {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}
