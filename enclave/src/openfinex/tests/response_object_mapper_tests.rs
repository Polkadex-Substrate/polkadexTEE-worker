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

use crate::openfinex::openfinex_types::{RequestType, ResponseInteger};
use crate::openfinex::response_object_mapper::{
    OpenFinexResponse, OpenFinexResponseObjectMapper, RequestResponse, ResponseObjectMapper,
};
use crate::openfinex::response_parser::{
    ParameterItem, ParameterNode, ParsedResponse, ResponseMethod,
};
use crate::openfinex::string_serialization::OpenFinexResponseDeserializer;
use crate::ss58check::account_id_to_ss58check;
use alloc::{string::String, string::ToString, sync::Arc, vec::Vec};
use codec::Encode;
use polkadex_sgx_primitives::types::{MarketId, OrderSide, OrderState, OrderType};
use polkadex_sgx_primitives::{accounts::get_account, AssetId};

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
            ParameterNode::SingleParameter(ParameterItem::String("admin".to_string())),
            ParameterNode::ParameterList(vec![
                ParameterItem::String("events.order".to_string()),
                ParameterItem::String("events.trade".to_string()),
            ]),
        ],
    };

    let mapped_objects = map_to_objects(&subscribe_response);

    match mapped_objects {
        OpenFinexResponse::RequestResponse(rr, rid) => match rr {
            RequestResponse::Subscription(sr) => {
                assert_eq!(rid, request_id);
                assert_eq!("admin".to_string(), sr.admin_name);
                assert_eq!(sr.subscribed_events.len(), 2);
                assert_eq!(
                    &"events.order".to_string(),
                    sr.subscribed_events.get(0).unwrap()
                );
                assert_eq!(
                    &"events.trade".to_string(),
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
    let order_id = "1245-2345-6798-123123".to_string();
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

pub fn test_order_cancel_response() {
    let order_uuid = "7acbbc84-939d-11eaa827-1831bf9834b0".to_string();
    let order_id = 2;

    let account = get_account("order_update_test_account");
    let account_nickname = account_id_to_ss58check(&account);

    let order_update_response = ParsedResponse {
        response_method: ResponseMethod::OrderCanceled,
        response_preamble: 5,
        parameters: vec![
            ParameterNode::SingleParameter(ParameterItem::String("ABC000001".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String(account_nickname)),
            ParameterNode::SingleParameter(ParameterItem::String("dotpdx".to_string())),
            ParameterNode::SingleParameter(ParameterItem::Number(order_id)),
            ParameterNode::SingleParameter(ParameterItem::String(order_uuid.clone())),
            ParameterNode::SingleParameter(ParameterItem::String("buy".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("d".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("l".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("1".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("2".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("3".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("4".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("5".to_string())),
            ParameterNode::SingleParameter(ParameterItem::Number(1)),
            ParameterNode::SingleParameter(ParameterItem::Number(1589211516)),
        ],
    };

    let mapped_objects = map_to_objects(&order_update_response);

    match mapped_objects {
        OpenFinexResponse::OrderCanceled(ou) => {
            assert_eq!(ou.order_id, order_id);
            assert_eq!(ou.unique_order_id, order_uuid.encode());
            assert_eq!(ou.user_id, account);
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

    let maker_account = get_account("trade_event_maker_test_account");
    let taker_account = get_account("trade_event_taker_test_account");
    let maker_nickname = account_id_to_ss58check(&maker_account);
    let taker_nickname = account_id_to_ss58check(&taker_account);

    let trade_event_response = ParsedResponse {
        response_method: ResponseMethod::TradeEvent,
        response_preamble: 5,
        parameters: vec![
            ParameterNode::SingleParameter(ParameterItem::String("pdxdot".to_string())),
            ParameterNode::SingleParameter(ParameterItem::Number(trade_id)),
            ParameterNode::SingleParameter(ParameterItem::String("1000".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("2".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String("2000".to_string())),
            ParameterNode::SingleParameter(ParameterItem::Number(2)),
            ParameterNode::SingleParameter(ParameterItem::String(
                "55d78eee-939e-11ea-945f-1831bf9834b0".to_string(),
            )),
            ParameterNode::SingleParameter(ParameterItem::String("A00001".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String(maker_nickname)),
            ParameterNode::SingleParameter(ParameterItem::Number(3)),
            ParameterNode::SingleParameter(ParameterItem::String(
                "55d78eee-939e-11ea-945f-1831bf9834as".to_string(),
            )),
            ParameterNode::SingleParameter(ParameterItem::String("A00002".to_string())),
            ParameterNode::SingleParameter(ParameterItem::String(taker_nickname)),
            ParameterNode::SingleParameter(ParameterItem::String("buy".to_string())),
            ParameterNode::SingleParameter(ParameterItem::Number(1589211884)),
        ],
    };

    let mapped_objects = map_to_objects(&trade_event_response);

    match mapped_objects {
        OpenFinexResponse::TradeEvent(te) => {
            assert_eq!(te.trade_id, trade_id);
            assert_eq!(te.funds, 2_000_000_000_000_000_000_000);
            assert_eq!(te.maker_order_id, 2);
            assert_eq!(te.taker_order_id, 3);
            assert_eq!(te.maker_user_id, maker_account);
            assert_eq!(te.taker_user_id, taker_account);
        }
        _ => {
            assert!(
                false,
                "Found unexpected response type, expected RequestResponse"
            );
        }
    }
}

pub fn test_get_markets_response() {
    let request_id: ResponseInteger = 4888721;
    let get_markets_response = ParsedResponse {
        response_method: ResponseMethod::FromRequestMethod(RequestType::GetMarkets, request_id),
        response_preamble: 2,
        parameters: vec![
            ParameterNode::SingleParameter(ParameterItem::Json(r#"{"param":"value"}"#.to_string())),
            ParameterNode::SingleParameter(ParameterItem::Json(r#"{"id":1234}"#.to_string())),
            ParameterNode::SingleParameter(ParameterItem::Json(r#"{}"#.to_string())),
        ],
    };

    let mapped_objects = map_to_objects(&get_markets_response);

    match mapped_objects {
        OpenFinexResponse::RequestResponse(rr, ri) => match rr {
            RequestResponse::GetMarkets(gmr) => {
                assert_eq!(ri, request_id);
                assert_eq!(gmr.json_content.len(), 3);
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

fn map_to_objects(response: &ParsedResponse) -> OpenFinexResponse {
    struct ResponseDeserializerMock;
    impl OpenFinexResponseDeserializer for ResponseDeserializerMock {
        fn string_to_market_id(&self, _market_id_str: &str) -> Result<MarketId, String> {
            Ok(MarketId {
                base: AssetId::POLKADEX,
                quote: AssetId::Asset(840),
            })
        }

        fn string_to_order_type(&self, _order_type_str: &str) -> Result<OrderType, String> {
            Ok(OrderType::LIMIT)
        }

        fn string_to_order_side(&self, _order_side_str: &str) -> Result<OrderSide, String> {
            Ok(OrderSide::BID)
        }

        fn string_to_order_state(&self, _order_state_str: &str) -> Result<OrderState, String> {
            Ok(OrderState::DONE)
        }

        fn string_to_asset_id(&self, _asset_id_str: &str) -> Result<AssetId, String> {
            Ok(AssetId::Asset(840))
        }
    }

    let object_mapper = ResponseObjectMapper::new(Arc::new(ResponseDeserializerMock {}));
    object_mapper.map_to_response_object(response).unwrap()
}
