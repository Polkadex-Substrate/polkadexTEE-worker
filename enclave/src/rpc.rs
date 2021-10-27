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

pub mod author;
pub mod error;

pub mod api;
pub mod basic_pool;
pub mod worker_api_direct;

pub mod io_handler_extensions;
pub mod return_value_encoding;
pub mod rpc_call_encoder;
pub mod rpc_info;

pub mod polkadex_rpc_gateway;
pub mod rpc_cancel_order;
pub mod rpc_edit_order;
pub mod rpc_get_balance;
pub mod rpc_nonce;
pub mod rpc_place_order;
pub mod rpc_withdraw;
pub mod trusted_operation_verifier;

pub mod mocks;
