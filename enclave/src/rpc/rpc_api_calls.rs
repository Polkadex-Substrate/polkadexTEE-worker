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
use log::*;
use substratee_node_primitives::Request;
use substratee_worker_primitives::DirectRequestStatus;

use crate::rpc::rpc_call::{RpcCall, RpcMethodImpl};
use crate::rpc::rpc_call_encoder::JsonRpcCallEncoder;

pub fn get_all_rpc_calls() -> Vec<RpcCall<JsonRpcCallEncoder, RpcMethodImpl>> {
    vec![
        RpcCall::new("place_order", &place_order, JsonRpcCallEncoder {}),
        RpcCall::new("cancel_order", &cancel_order, JsonRpcCallEncoder {}),
        RpcCall::new("withdraw", &withdraw, JsonRpcCallEncoder {}),
        RpcCall::new("get_balance", &get_balance, JsonRpcCallEncoder {}),
    ]
}

fn place_order(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    debug!("entering place_order RPC");

    // TODO call implementation here

    Ok(("called place_order", false, DirectRequestStatus::Ok))
}

fn cancel_order(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    debug!("entering cancel_order RPC");

    // TODO call implementation here

    Ok(("called cancel_order", false, DirectRequestStatus::Ok))
}

fn withdraw(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    debug!("entering withdraw RPC");

    // TODO call implementation here

    Ok(("called withdraw", false, DirectRequestStatus::Ok))
}

pub fn get_balance(_request: Request) -> RpcResult<(&'static str, bool, DirectRequestStatus)> {
    debug!("entering get_balance RPC");

    // TODO call implementation here

    Ok(("called get_balance", false, DirectRequestStatus::Ok))
}
