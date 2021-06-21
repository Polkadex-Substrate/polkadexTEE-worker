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
use crate::openfinex::client_utils::Payload;
use crate::openfinex::openfinex_types::RequestId;
use crate::openfinex::response_object_mapper::{
    OpenFinexResponse, OpenFinexResponseObjectMapper, RequestResponse,
};
use crate::openfinex::response_parser::{ParsedResponse, ResponseParser};
use crate::polkadex_gateway::PolkaDexGatewayCallback;
use alloc::sync::Arc;
use log::*;
use polkadex_sgx_primitives::types::{OrderState, OrderUpdate, TradeEvent};

/// Trait for handling TCP responses, as they are received in the OpenFinex client
pub trait TcpResponseHandler {
    fn handle_text_op(&self, payload: Payload);
}

/// implementation for handling TCP responses from OpenFinex and processing them in the Polkadex Gateway
pub struct PolkadexResponseHandler {
    callback: Arc<dyn PolkaDexGatewayCallback>,
    response_parser: Arc<dyn ResponseParser>,
    response_object_mapper: Arc<dyn OpenFinexResponseObjectMapper>,
}

impl TcpResponseHandler for PolkadexResponseHandler {
    fn handle_text_op(&self, payload: Payload) {
        match payload {
            Payload::Text(s) => match self.response_parser.parse_response_string(s) {
                Ok(r) => self.handle_response(r),
                Err(e) => error!("Failed to parse TCP response string: {}", e),
            },
            _ => error!("Expected text payload, cannot handle TCP response, aborting"),
        }
    }
}

impl PolkadexResponseHandler {
    pub fn new(
        callback: Arc<dyn PolkaDexGatewayCallback>,
        response_parser: Arc<dyn ResponseParser>,
        response_object_mapper: Arc<dyn OpenFinexResponseObjectMapper>,
    ) -> Self {
        PolkadexResponseHandler {
            callback,
            response_parser,
            response_object_mapper,
        }
    }

    fn handle_response(&self, response: ParsedResponse) {
        // map response to an object model
        let response_objects = match self
            .response_object_mapper
            .map_to_response_object(&response)
        {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to map response to objects: {}", e);
                return;
            }
        };

        match response_objects {
            OpenFinexResponse::RequestResponse(request_response, request_id) => {
                self.handle_request_response(request_response, request_id)
            }
            OpenFinexResponse::Error(description) => {
                error!("OpenFinex reports an error: {}", description)
            }
            OpenFinexResponse::OrderUpdate(order_update) => self.handle_order_update(order_update),
            OpenFinexResponse::TradeEvent(trade_event) => self.handle_trade_event(trade_event),
        }
    }

    fn handle_order_update(&self, order_update: OrderUpdate) {
        debug!("Received order update from OpenFinex");

        match order_update.state {
            // cancel order
            OrderState::CANCEL => match self
                .callback
                .process_cancel_order(order_update.unique_order_id)
            {
                Ok(_) => {
                    debug!("Cancelling order succeeded")
                }
                Err(e) => {
                    error!("Cancelling order failed: {}", e)
                }
            },
            _ => {}
        }
    }

    fn handle_trade_event(&self, _trade_event: TradeEvent) {
        debug!("Received trade event from OpenFinex");
    }

    fn handle_request_response(&self, request_response: RequestResponse, _request_id: RequestId) {
        match request_response {
            RequestResponse::CreateOrder(cr) => {
                debug!("Received a create order response from OpenFinex");
                match self.callback.process_create_order(cr.order_id) {
                    Ok(_) => {
                        debug!("Creating order succeeded")
                    }
                    Err(e) => {
                        error!("Creating order failed: {}", e)
                    }
                }
            }
            RequestResponse::Subscription(_sr) => {
                debug!("Received a subscription response from OpenFinex");
            }
            RequestResponse::DepositFunds(_dr) => {
                debug!("Received a deposit funds response from OpenFinex");
            }
            RequestResponse::WithdrawFunds(_wr) => {
                debug!("Received a withdraw funds response from OpenFinex");
            }
        }
    }
}
