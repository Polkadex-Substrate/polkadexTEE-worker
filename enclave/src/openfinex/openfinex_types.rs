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
use crate::openfinex::openfinex_api::{OpenFinexApiError, OpenFinexApiResult};
use alloc::{fmt::Result as FormatResult, string::String};
use sp_runtime::FixedU128;

pub type Preamble = u128;
pub type RequestId = u128;
pub type ResponseInteger = u128;
pub type OpenFinexDecimal = FixedU128;

#[derive(Debug, Clone, PartialEq)]
pub enum RequestType {
    DepositFunds,
    WithdrawFunds,
    CreateOrder,
    CancelOrder,
    Subscribe,
}

impl alloc::fmt::Display for RequestType {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> FormatResult {
        write!(f, "{:?}", self)
    }
}

impl RequestType {
    const DEPOSIT_STR: &'static str = "admin_deposit";
    const WITHDRAW_STR: &'static str = "admin_withdraw";
    const CREATE_ORDER_STR: &'static str = "admin_create_order";
    const CANCEL_ORDER_STR: &'static str = "admin_cancel_order";
    const SUBSCRIBE_EVENTS_STR: &'static str = "subscribe";

    pub fn to_request_string(&self) -> String {
        match &self {
            RequestType::DepositFunds => String::from(RequestType::DEPOSIT_STR),
            RequestType::WithdrawFunds => String::from(RequestType::WITHDRAW_STR),
            RequestType::CreateOrder => String::from(RequestType::CREATE_ORDER_STR),
            RequestType::CancelOrder => String::from(RequestType::CANCEL_ORDER_STR),
            RequestType::Subscribe => String::from(RequestType::SUBSCRIBE_EVENTS_STR),
        }
    }

    pub fn from_request_string(input: &String) -> OpenFinexApiResult<Self> {
        match input.as_str() {
            RequestType::DEPOSIT_STR => Ok(RequestType::DepositFunds),
            RequestType::WITHDRAW_STR => Ok(RequestType::WithdrawFunds),
            RequestType::CREATE_ORDER_STR => Ok(RequestType::CreateOrder),
            RequestType::CANCEL_ORDER_STR => Ok(RequestType::CancelOrder),
            RequestType::SUBSCRIBE_EVENTS_STR => Ok(RequestType::Subscribe),
            _ => Err(OpenFinexApiError::ResponseParsingError(format!(
                "invalid method string {}",
                input
            ))),
        }
    }
}
