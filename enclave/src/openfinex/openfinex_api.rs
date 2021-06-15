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
use crate::openfinex::openfinex_types::RequestId;
use alloc::{fmt::Result as FormatResult, string::String};
use polkadex_sgx_primitives::types::{CancelOrder, Order};

/// error type for OpenFinex API calls
#[derive(Debug, Eq, PartialEq, PartialOrd)]
pub enum OpenFinexApiError {
    /// Error in serializing domain objects to string
    SerializationError(String),

    /// Error communicating via web socket
    WebSocketError(String),

    /// Error when parsing response
    ResponseParsingError(String),

    /// Errors related to the conversion between floating points and integers
    FixedPointConversionError(String),
}

impl alloc::fmt::Display for OpenFinexApiError {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> FormatResult {
        write!(f, "{:?}", self)
    }
}

/// result type definition using the OpenFinexApiError
pub type OpenFinexApiResult<T> = Result<T, OpenFinexApiError>;

/// OpenFinex API trait
pub trait OpenFinexApi {
    fn create_order(&self, order: Order, request_id: RequestId) -> OpenFinexApiResult<()>;

    fn cancel_order(
        &self,
        cancel_order: CancelOrder,
        request_id: RequestId,
    ) -> OpenFinexApiResult<()>;

    fn withdraw_funds(&self, request_id: RequestId) -> OpenFinexApiResult<()>;

    fn deposit_funds(&self, request_id: RequestId) -> OpenFinexApiResult<()>;
}
