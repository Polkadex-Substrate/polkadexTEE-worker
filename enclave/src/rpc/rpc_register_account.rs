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
use alloc::{boxed::Box, string::String, string::ToString};

use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::RpcCallStatus;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;

use crate::rpc::polkadex_rpc_gateway::RpcGateway;
use crate::rpc::trusted_operation_verifier::TrustedOperationExtractor;
use polkadex_sgx_primitives::types::DirectRequest;
use substratee_stf::{TrustedCall, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcRegisterAccount {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,

    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcRegisterAccount {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcRegisterAccount {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> Result<((), bool, DirectRequestStatus), String> {
        debug!("entering register_account RPC");

        let verified_trusted_operation =
            self.top_extractor.get_verified_trusted_operation(request)?;

        let trusted_call = self
            .rpc_gateway
            .authorize_trusted_call(verified_trusted_operation)?;

        match trusted_call {
            TrustedCall::register_account(_, _) => (),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }

        let main_account = trusted_call.main_account().clone();
        let proxy_account = trusted_call.proxy_account();

        match self
            .rpc_gateway
            .register_account(main_account, proxy_account.unwrap_or(main_account.clone()))
        {
            Ok(()) => Ok(((), false, DirectRequestStatus::Ok)),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl RpcCall for RpcRegisterAccount {
    fn name() -> String {
        "register_account".to_string()
    }
}

impl RpcMethodSync for RpcRegisterAccount {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}
