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
use crate::rpc::polkadex_rpc_gateway::RpcGateway;
use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::RpcCallStatus;
use crate::rpc::trusted_operation_verifier::TrustedOperationExtractor;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use polkadex_sgx_primitives::types::DirectRequest;
use substratee_stf::{Getter, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcGetNonce {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcGetNonce {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcGetNonce {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(
        &self,
        request: DirectRequest,
    ) -> Result<(u32, bool, DirectRequestStatus), String> {
        debug!("entering get_balance RPC");

        let verified_trusted_operation =
            self.top_extractor.get_verified_trusted_operation(request)?;

        let trusted_getter_signed = match verified_trusted_operation {
            TrustedOperation::get(getter) => match getter {
                Getter::trusted(tgs) => Ok(tgs),
                _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
            },
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let main_account = trusted_getter_signed.getter.main_account().clone();
        let proxy_account = trusted_getter_signed.getter.proxy_account().clone();

        let _authorization_result = match self
            .rpc_gateway
            .authorize_user(main_account.clone(), proxy_account)
        {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Authorization error: {}", e)),
        }?;

        let nonce = match self
            .rpc_gateway
            .get_nonce(main_account.clone())
        {
            Ok(b) => Ok(b.nonce.unwrap()), //TODO: Fix error handling
            Err(e) => Err(String::from(e.as_str())),
        }?;

        debug!("Nonce: {:?}", nonce);

        Ok((nonce, false, DirectRequestStatus::Ok))
    }
}

impl RpcCall for RpcGetNonce {
    fn name() -> String {
        "get_nonce".to_string()
    }
}

impl RpcMethodSync for RpcGetNonce {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: DirectRequest| self.method_impl(r))
    }
}

pub mod tests {

    pub extern crate alloc;
    use crate::rpc::mocks::dummy_builder::{create_dummy_account, create_dummy_request};
    use crate::rpc::mocks::{
        rpc_gateway_mock::RpcGatewayMock,
        trusted_operation_extractor_mock::TrustedOperationExtractorMock,
    };
    use crate::rpc::rpc_get_nonce::RpcGetNonce;
    use alloc::boxed::Box;
    use sp_core::Pair;
    use substratee_stf::{Getter, KeyPair, TrustedGetter, TrustedOperation};

    pub fn test_given_valid_top_return_nonce() {
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_get_nonce_getter()),
        });
        let request = create_dummy_request();

        let rpc_gateway = Box::new(RpcGatewayMock::mock_nonce(true));

        let rpc_get_nonce = RpcGetNonce::new(top_extractor, rpc_gateway);

        let result = rpc_get_nonce.method_impl(request).unwrap();
        assert_eq!(result.0, 0);
    }

    fn create_get_nonce_getter() -> TrustedOperation {
        let key_pair = create_dummy_account();

        let trusted_getter =
            TrustedGetter::nonce(key_pair.public().into());
        let trusted_getter_signed = trusted_getter.sign(&KeyPair::Ed25519(key_pair));

        TrustedOperation::get(Getter::trusted(trusted_getter_signed))
    }
}
