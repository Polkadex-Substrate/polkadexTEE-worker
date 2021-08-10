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
use crate::openfinex::fixed_point_number_converter::FixedPointNumberConverter;
use crate::openfinex::openfinex_api::{OpenFinexApi, OpenFinexApiError, OpenFinexApiResult};
use crate::openfinex::openfinex_client::OpenFinexClientInterface;
use crate::openfinex::openfinex_types::{RequestId, RequestType};
use crate::openfinex::request_builder::OpenFinexRequestBuilder;
use crate::openfinex::string_serialization::{
    market_id_to_request_string, market_type_to_request_string, order_side_to_request_string,
    order_type_to_request_string, order_uuid_to_request_string, user_id_to_request_string,
};
use log::*;
use polkadex_sgx_primitives::types::{CancelOrder, Order};

/// implementation of the OpenFinex API
pub struct OpenFinexApiImpl {
    websocket_client: OpenFinexClientInterface,
}

impl OpenFinexApiImpl {
    pub fn new(websocket_client: OpenFinexClientInterface) -> Self {
        OpenFinexApiImpl { websocket_client }
    }

    fn create_builder(
        &self,
        request_type: RequestType,
        request_id: RequestId,
    ) -> OpenFinexRequestBuilder {
        OpenFinexRequestBuilder::new(request_type, request_id)
    }
}

/// implementation
impl OpenFinexApi for OpenFinexApiImpl {
    fn create_order(&self, order: Order, request_id: RequestId) -> OpenFinexApiResult<RequestId> {
        let user_id = user_id_to_request_string(&order.user_uid);
        let market_type = market_type_to_request_string(order.market_type)?;
        let order_type = order_type_to_request_string(order.order_type);
        let order_side = order_side_to_request_string(order.side);

        let quantity_decimal = FixedPointNumberConverter::_to_string(order.quantity);
        let price_decimal = order.price.map(FixedPointNumberConverter::_to_string);

        let request = self
            .create_builder(RequestType::CreateOrder, request_id)
            .push_optional_parameter(None) // empty parameter for uid
            .push_optional_parameter(Some(user_id)) // nickname
            .push_parameter(market_id_to_request_string(order.market_id))
            .push_parameter(market_type)
            .push_parameter(order_type)
            .push_parameter(order_side)
            .push_parameter(quantity_decimal)
            .push_optional_parameter(price_decimal)
            .build();
        debug!(
            "Sending order to openfinex: {}",
            request.to_request_string()
        );

        self.websocket_client
            .clone()
            .send_request(&request.to_request_string().as_bytes())
            .map_err(|e| OpenFinexApiError::WebSocketError(format!("{:?}", e)))?;
        Ok(request_id)
    }

    fn cancel_order(
        &self,
        cancel_order: CancelOrder,
        request_id: RequestId,
    ) -> OpenFinexApiResult<()> {
        // FIXME: Currently only one order_id support. We will need to change that
        let order_id = order_uuid_to_request_string(cancel_order.order_id)?;
        let request = self
            .create_builder(RequestType::CancelOrder, request_id)
            .push_parameter(market_id_to_request_string(cancel_order.market_id))
            .push_list_parameter(vec![order_id])
            .build();
        debug!(
            "Sending order to openfinex: {}",
            request.to_request_string()
        );
        self.websocket_client
            .clone()
            .send_request(&request.to_request_string().as_bytes())
            .map_err(|e| OpenFinexApiError::WebSocketError(format!("{:?}", e)))
    }

    fn withdraw_funds(&self, _request_id: RequestId) -> OpenFinexApiResult<()> {
        todo!()
    }

    fn deposit_funds(&self, _request_id: RequestId) -> OpenFinexApiResult<()> {
        todo!()
    }
}
