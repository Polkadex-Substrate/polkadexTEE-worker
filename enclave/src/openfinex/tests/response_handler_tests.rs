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

use crate::openfinex::openfinex_api::OpenFinexApiResult;
use crate::openfinex::response_handler::PolkadexResponseHandler;
use crate::openfinex::response_object_mapper::{OpenFinexResponse, OpenFinexResponseObjectMapper};
use crate::openfinex::response_parser::{ParsedResponse, ResponseParser};
use crate::polkadex_gateway::{GatewayError, PolkaDexGatewayCallback};
use alloc::{string::String, sync::Arc};
use polkadex_sgx_primitives::types::OrderUUID;

struct GatewayCallBackMock;
impl PolkaDexGatewayCallback for GatewayCallBackMock {
    fn process_cancel_order(&self, order_uuid: OrderUUID) -> Result<(), GatewayError> {
        todo!()
    }

    fn process_create_order(&self, order_uuid: OrderUUID) -> Result<(), GatewayError> {
        todo!()
    }
}

struct ResponseParserMock;
impl ResponseParser for ResponseParserMock {
    fn parse_response_string(&self, response: String) -> OpenFinexApiResult<ParsedResponse> {
        todo!()
    }
}

struct ResponseObjectMapperMock;
impl OpenFinexResponseObjectMapper for ResponseObjectMapperMock {
    fn map_to_response_object(
        &self,
        parsed_response: &ParsedResponse,
    ) -> OpenFinexApiResult<OpenFinexResponse> {
        todo!()
    }
}

fn create_response_handler() -> PolkadexResponseHandler {
    PolkadexResponseHandler::new(
        Arc::new(GatewayCallBackMock {}),
        Arc::new(ResponseParserMock {}),
        Arc::new(ResponseObjectMapperMock {}),
    )
}
