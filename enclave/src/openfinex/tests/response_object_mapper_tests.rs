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

use crate::openfinex::openfinex_types::{RequestType, ResponseInteger};
use crate::openfinex::response_object_mapper::{
    OpenFinexResponse, OpenFinexResponseObjectMapper, RequestResponse, ResponseObjectMapper,
};
use crate::openfinex::response_parser::{
    ParameterItem, ParameterNode, ParsedResponse, ResponseMethod,
};
use alloc::{string::String, vec::Vec};
use codec::Encode;
use polkadex_sgx_primitives::types::{MarketId, OrderState, OrderType};
use polkadex_sgx_primitives::AssetId;

pub fn test_given_parsed_error_then_map_to_error_object() {
    let error_description = String::from("error description");
    let error_response = ParsedResponse {
        response_method: ResponseMethod::Error(error_description.clone()),
        response_preamble: 2,
        parameters: Vec::new(),
    };

    let mapped_objects = map_to_objects(&error_response);

    match mapped_objects {
        OpenFinexResponse::Error(s) => {
            assert_eq!(s, error_description);
        }
        _ => {
            assert!(false, "Found unexpected response type, expected error");
        }
    }
}

pub fn test_subscribe_response() {
    let request_id: ResponseInteger = 9872345214;
    let subscribe_response = ParsedResponse {
        response_method: ResponseMethod::FromRequestMethod(RequestType::Subscribe, request_id),
        response_preamble: 51,
        parameters: vec![
            ParameterNode::SingleParameter(ParameterItem::String(format!("admin"))),
            ParameterNode::ParameterList(vec![
                ParameterItem::String(format!("events.order")),
                ParameterItem::String(format!("events.trade")),
            ]),
        ],
    };

    let mapped_objects = map_to_objects(&subscribe_response);

    match mapped_objects {
        OpenFinexResponse::RequestResponse(rr, rid) => match rr {
            RequestResponse::Subscription(sr) => {
                assert_eq!(rid, request_id);
                assert_eq!(format!("admin"), sr.admin_name);
                assert_eq!(sr.subscribed_events.len(), 2);
                assert_eq!(
                    &format!("events.order"),
                    sr.subscribed_events.get(0).unwrap()
                );
                assert_eq!(
                    &format!("events.trade"),
                    sr.subscribed_events.get(1).unwrap()
                );
            }
            _ => {
                assert!(false, "Found unexpected RequestResponse");
            }
        },
        _ => {
            assert!(
                false,
                "Found unexpected response type, expected RequestResponse"
            );
        }
    }
}

pub fn test_create_order_response() {
    let request_id: ResponseInteger = 12;
    let order_id = format!("1245-2345-6798-123123");
    let subscribe_response = ParsedResponse {
        response_method: ResponseMethod::FromRequestMethod(RequestType::CreateOrder, request_id),
        response_preamble: 2,
        parameters: vec![ParameterNode::SingleParameter(ParameterItem::String(
            order_id.clone(),
        ))],
    };

    let mapped_objects = map_to_objects(&subscribe_response);

    match mapped_objects {
        OpenFinexResponse::RequestResponse(rr, rid) => match rr {
            RequestResponse::CreateOrder(cr) => {
                assert_eq!(rid, request_id);
                assert_eq!(order_id.encode(), cr.order_id);
            }
            _ => {
                assert!(false, "Found unexpected RequestResponse");
            }
        },
        _ => {
            assert!(
                false,
                "Found unexpected response type, expected RequestResponse"
            );
        }
    }
}

pub fn test_order_update_response() {
    let order_uuid = format!("7acbbc84-939d-11eaa827-1831bf9834b0");
    let order_id = 2;

    let order_update_response = ParsedResponse {
        response_method: ResponseMethod::OrderUpdate,
        response_preamble: 5,
        parameters: vec![
            ParameterNode::SingleParameter(ParameterItem::String(format!("ABC000001"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("0x1234567890123456789"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("dotpdx"))),
            ParameterNode::SingleParameter(ParameterItem::Number(order_id)),
            ParameterNode::SingleParameter(ParameterItem::String(order_uuid.clone())),
            ParameterNode::SingleParameter(ParameterItem::String(format!("buy"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("d"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("l"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("1"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("2"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("3"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("4"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("5"))),
            ParameterNode::SingleParameter(ParameterItem::Number(1)),
            ParameterNode::SingleParameter(ParameterItem::Number(1589211516)),
        ],
    };

    let mapped_objects = map_to_objects(&order_update_response);

    match mapped_objects {
        OpenFinexResponse::OrderUpdate(ou) => {
            assert_eq!(
                ou.market_id,
                MarketId {
                    base: AssetId::DOT,
                    quote: AssetId::POLKADEX
                }
            );

            assert_eq!(ou.order_id, order_id);
            assert_eq!(ou.unique_order_id, order_uuid.encode());
            assert_eq!(ou.state, OrderState::DONE);
            assert_eq!(ou.order_type, OrderType::LIMIT);
        }
        _ => {
            assert!(
                false,
                "Found unexpected response type, expected RequestResponse"
            );
        }
    }
}

pub fn test_trade_event_response() {
    let trade_id = 98725621;

    let trade_event_response = ParsedResponse {
        response_method: ResponseMethod::TradeEvent,
        response_preamble: 5,
        parameters: vec![
            ParameterNode::SingleParameter(ParameterItem::String(format!("pdxdot"))),
            ParameterNode::SingleParameter(ParameterItem::Number(trade_id)),
            ParameterNode::SingleParameter(ParameterItem::String(format!("1000"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("2"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("2000"))),
            ParameterNode::SingleParameter(ParameterItem::Number(2)),
            ParameterNode::SingleParameter(ParameterItem::String(format!(
                "55d78eee-939e-11ea-945f-1831bf9834b0"
            ))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("A00001"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("0x000001"))),
            ParameterNode::SingleParameter(ParameterItem::Number(3)),
            ParameterNode::SingleParameter(ParameterItem::String(format!(
                "55d78eee-939e-11ea-945f-1831bf9834as"
            ))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("A00002"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("0x000002"))),
            ParameterNode::SingleParameter(ParameterItem::String(format!("buy"))),
            ParameterNode::SingleParameter(ParameterItem::Number(1589211884)),
        ],
    };

    let mapped_objects = map_to_objects(&trade_event_response);

    match mapped_objects {
        OpenFinexResponse::TradeEvent(te) => {
            assert_eq!(
                te.market_id,
                MarketId {
                    base: AssetId::POLKADEX,
                    quote: AssetId::DOT
                }
            );

            assert_eq!(te.trade_id, trade_id);
            assert_eq!(te.funds, 2_000_000_000_000_000_000_000);
            assert_eq!(te.maker_order_id, 2);
            assert_eq!(te.taker_order_id, 3);
        }
        _ => {
            assert!(
                false,
                "Found unexpected response type, expected RequestResponse"
            );
        }
    }
}

fn map_to_objects(response: &ParsedResponse) -> OpenFinexResponse {
    let object_mapper = ResponseObjectMapper {};
    object_mapper.map_to_response_object(response).unwrap()
}
