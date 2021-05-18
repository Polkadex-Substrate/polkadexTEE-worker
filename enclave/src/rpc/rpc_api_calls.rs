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
use alloc::vec::Vec;

use jsonrpc_core::Result as RpcResult;

use substratee_node_primitives::Request;

use crate::rpc::rpc_call::{RpcCall, RpcMethodImpl};
use crate::rpc::rpc_call_encoder::JsonRpcCallEncoder;
use substratee_worker_primitives::DirectRequestStatus;

pub fn get_all_rpc_calls() -> Vec<RpcCall<JsonRpcCallEncoder, RpcMethodImpl>> {
    vec![
        RpcCall::new("place_order", &place_order, JsonRpcCallEncoder {}),
        RpcCall::new("cancel_order", &cancel_order, JsonRpcCallEncoder {}),
        RpcCall::new("withdraw", &withdraw, JsonRpcCallEncoder {}),
        RpcCall::new("get_balance", &get_balance, JsonRpcCallEncoder {}),
        RpcCall::new(
            "subscribe_matches",
            &subscribe_matches,
            JsonRpcCallEncoder {},
        ),
    ]
}

fn place_order(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    Ok(("called place_order", false, DirectRequestStatus::Ok))
}

fn cancel_order(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    Ok(("called cancel_order", false, DirectRequestStatus::Ok))
}

fn withdraw(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    Ok(("called withdraw", false, DirectRequestStatus::Ok))
}

pub fn get_balance(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    Ok(("called get_balance", false, DirectRequestStatus::Ok))
}

fn subscribe_matches(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    Ok(("called subscribe_matches", false, DirectRequestStatus::Ok))
}
