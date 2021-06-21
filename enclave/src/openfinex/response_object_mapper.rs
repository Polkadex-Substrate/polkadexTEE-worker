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
use crate::openfinex::fixed_point_number_converter::FixedPointNumberConverter;
use crate::openfinex::openfinex_api::{OpenFinexApiError, OpenFinexApiResult};
use crate::openfinex::openfinex_types::{RequestId, RequestType, ResponseInteger};
use crate::openfinex::response_parser::{
    ParameterItem, ParameterNode, ParsedResponse, ResponseMethod,
};
use crate::openfinex::string_serialization::OpenFinexResponseDeserializer;
use crate::ss58check::ss58check_to_account_id;
use alloc::{string::String, sync::Arc, vec::Vec};
use codec::Encode;
use core::iter::Peekable;
use polkadex_sgx_primitives::types::{OrderUUID, OrderUpdate, PriceAndQuantityType, TradeEvent};

/// OpenFinex Response Root Node
#[derive(Debug, Clone, PartialEq)]
pub enum OpenFinexResponse {
    RequestResponse(RequestResponse, RequestId),
    OrderUpdate(OrderUpdate),
    TradeEvent(TradeEvent),
    Error(String),
}

/// Response to a request
#[derive(Debug, Clone, PartialEq)]
pub enum RequestResponse {
    DepositFunds(DepositResponse),
    WithdrawFunds(WithdrawResponse),
    CreateOrder(CreateOrderResponse),
    Subscription(SubscriptionResponse),
    GetMarkets(GetMarketsResponse),
}

/// Deposit funds response
#[derive(Debug, Clone, PartialEq)]
pub struct DepositResponse {
    pub db_record_id: ResponseInteger,
}

/// Withdraw funds response
#[derive(Debug, Clone, PartialEq)]
pub struct WithdrawResponse {
    pub db_record_id: ResponseInteger,
}

/// Create order response
#[derive(Debug, Clone, PartialEq)]
pub struct CreateOrderResponse {
    pub order_id: OrderUUID,
}

/// Subscription to events response
#[derive(Debug, Clone, PartialEq)]
pub struct SubscriptionResponse {
    pub admin_name: String, // TODO this is not exactly clear what it means
    pub subscribed_events: Vec<String>,
}

/// Response to requesting markets information
#[derive(Debug, Clone, PartialEq)]
pub struct GetMarketsResponse {
    pub json_content: Vec<String>, // strings containing JSON objects
}

pub trait OpenFinexResponseObjectMapper {
    fn map_to_response_object(
        &self,
        parsed_response: &ParsedResponse,
    ) -> OpenFinexApiResult<OpenFinexResponse>;
}

pub struct ResponseObjectMapper {
    string_deserializer: Arc<dyn OpenFinexResponseDeserializer>,
}

impl OpenFinexResponseObjectMapper for ResponseObjectMapper {
    fn map_to_response_object(
        &self,
        parsed_response: &ParsedResponse,
    ) -> OpenFinexApiResult<OpenFinexResponse> {
        match &parsed_response.response_method {
            ResponseMethod::Error(s) => Ok(OpenFinexResponse::Error(s.clone())),
            ResponseMethod::FromRequestMethod(rt, ri) => {
                self.map_request_response(rt, ri, &parsed_response.parameters)
            }
            ResponseMethod::TradeEvent => self.map_trade_event(&parsed_response.parameters),
            ResponseMethod::OrderUpdate => self.map_order_update(&parsed_response.parameters),
        }
    }
}

impl ResponseObjectMapper {
    pub fn new(string_deserializer: Arc<dyn OpenFinexResponseDeserializer>) -> Self {
        ResponseObjectMapper {
            string_deserializer,
        }
    }

    fn map_request_response(
        &self,
        request_type: &RequestType,
        request_id: &RequestId,
        parameters: &Vec<ParameterNode>,
    ) -> OpenFinexApiResult<OpenFinexResponse> {
        let mut param_iter = parameters.iter().peekable();

        match request_type {
            RequestType::CreateOrder => {
                let order_id =
                    get_next_single_item(&mut param_iter, &extract_encoded_string_from_item)?;
                Ok(OpenFinexResponse::RequestResponse(
                    RequestResponse::CreateOrder(CreateOrderResponse { order_id }),
                    *request_id,
                ))
            }
            RequestType::WithdrawFunds => {
                let db_record_id =
                    get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
                Ok(OpenFinexResponse::RequestResponse(
                    RequestResponse::WithdrawFunds(WithdrawResponse { db_record_id }),
                    *request_id,
                ))
            }
            RequestType::DepositFunds => {
                let db_record_id =
                    get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
                Ok(OpenFinexResponse::RequestResponse(
                    RequestResponse::DepositFunds(DepositResponse { db_record_id }),
                    *request_id,
                ))
            }
            RequestType::Subscribe => {
                let admin_name = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
                let subscribed_events =
                    get_next_item_list(&mut param_iter, &extract_string_from_item)?;
                Ok(OpenFinexResponse::RequestResponse(
                    RequestResponse::Subscription(SubscriptionResponse {
                        admin_name,
                        subscribed_events,
                    }),
                    *request_id,
                ))
            }
            RequestType::GetMarkets => {
                let json_string_objects = get_next_items(&mut param_iter, &extract_json_from_item)?;
                Ok(OpenFinexResponse::RequestResponse(
                    RequestResponse::GetMarkets(GetMarketsResponse {
                        json_content: json_string_objects,
                    }),
                    *request_id,
                ))
            }
            _ => Err(OpenFinexApiError::ResponseParsingError(format!(
                "Unknown or unsupported request type ({}), cannot map to response",
                request_type
            ))),
        }
    }

    fn map_trade_event(
        &self,
        parameters: &Vec<ParameterNode>,
    ) -> OpenFinexApiResult<OpenFinexResponse> {
        let mut param_iter = parameters.iter().peekable();

        let market_id_str = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let trade_id = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
        let price = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;
        let amount = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;
        let funds = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;

        let maker_order_id = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
        let maker_order_uuid =
            get_next_single_item(&mut param_iter, &extract_encoded_string_from_item)?;
        let _maker_uid = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let maker_nickname = get_next_single_item(&mut param_iter, &extract_string_from_item)?;

        let taker_order_id = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
        let taker_order_uuid =
            get_next_single_item(&mut param_iter, &extract_encoded_string_from_item)?;
        let _taker_uid = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let taker_nickname = get_next_single_item(&mut param_iter, &extract_string_from_item)?;

        let maker_order_side_str =
            get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let timestamp = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;

        let market_id = self
            .string_deserializer
            .string_to_market_id(&market_id_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let maker_side = self
            .string_deserializer
            .string_to_order_side(&maker_order_side_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        // we use the nickname to pass our user ID currently, see the request side
        let maker_user_id = ss58check_to_account_id(maker_nickname.as_str())
            .map_err(|e| OpenFinexApiError::ResponseParsingError(format!("{}", e)))?;

        let taker_user_id = ss58check_to_account_id(taker_nickname.as_str())
            .map_err(|e| OpenFinexApiError::ResponseParsingError(format!("{}", e)))?;

        Ok(OpenFinexResponse::TradeEvent(TradeEvent {
            market_id,
            trade_id,
            price,
            amount,
            funds,
            maker_user_id,
            maker_order_id,
            maker_order_uuid,
            taker_user_id,
            taker_order_id,
            taker_order_uuid,
            maker_side,
            timestamp,
        }))
    }

    fn map_order_update(
        &self,
        parameters: &Vec<ParameterNode>,
    ) -> OpenFinexApiResult<OpenFinexResponse> {
        let mut param_iter = parameters.iter().peekable();

        let _user_identifier = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let user_nickname_str = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let market_id_str = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let order_id = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
        let order_uuid = get_next_single_item(&mut param_iter, &extract_encoded_string_from_item)?;
        let order_side_str = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let order_state_str = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let order_type_str = get_next_single_item(&mut param_iter, &extract_string_from_item)?;

        let price = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;
        let price_average = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;
        let volume_order = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;
        let volume_origin = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;
        let volume_executed = get_next_single_item(&mut param_iter, &extract_decimal_from_item)?;

        let trades_count = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
        let timestamp = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;

        let market_id = self
            .string_deserializer
            .string_to_market_id(&market_id_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let order_side = self
            .string_deserializer
            .string_to_order_side(&order_side_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let order_type = self
            .string_deserializer
            .string_to_order_type(&order_type_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let order_state = self
            .string_deserializer
            .string_to_order_state(&order_state_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        // user ID is taken from nickname - has to match what we compose in the request, see
        // openfinex_api_impl.rs
        let user_id = ss58check_to_account_id(user_nickname_str.as_str())
            .map_err(|e| OpenFinexApiError::ResponseParsingError(format!("{}", e)))?;

        Ok(OpenFinexResponse::OrderUpdate(OrderUpdate {
            user_id,
            market_id,
            order_id,
            unique_order_id: order_uuid,
            side: order_side,
            state: order_state,
            order_type,
            price,
            avg_price: price_average,
            order_volume: volume_order,
            original_volume: volume_origin,
            executed_volume: volume_executed,
            trade_count_order: trades_count,
            timestamp,
        }))
    }
}

fn get_next_single_item<'a, T: Iterator<Item = &'a ParameterNode>, R>(
    iter: &mut Peekable<T>,
    item_extractor: &dyn Fn(&ParameterItem) -> OpenFinexApiResult<R>,
) -> OpenFinexApiResult<R> {
    match iter.next() {
        Some(p) => {
            let parameter_item = extract_single_parameter_from_node(p)?;
            (item_extractor)(parameter_item)
        }
        None => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a parameter node, but list of parameter nodes ended prematurely"
        ))),
    }
}

fn get_next_item_list<'a, T: Iterator<Item = &'a ParameterNode>, R>(
    iter: &mut Peekable<T>,
    item_extractor: &dyn Fn(&ParameterItem) -> OpenFinexApiResult<R>,
) -> OpenFinexApiResult<Vec<R>> {
    match iter.next() {
        Some(p) => {
            let parameter_items = extract_nested_parameter_list_from_node(p)?;

            // this is a shortcut: we take all the items that we can extract successfully and discard everything else
            let extracted_items = parameter_items
                .iter()
                .map(|i| (item_extractor)(i))
                .filter_map(|l| l.ok())
                .collect::<Vec<R>>();

            Ok(extracted_items)
        }
        None => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a parameter node, but list of parameter nodes ended prematurely"
        ))),
    }
}

/// get as many items that match the extractor in a row
fn get_next_items<'a, T: Iterator<Item = &'a ParameterNode>, R>(
    iter: &mut Peekable<T>,
    item_extractor: &dyn Fn(&ParameterItem) -> OpenFinexApiResult<R>,
) -> OpenFinexApiResult<Vec<R>> {
    let mut matched_items = Vec::<R>::new();
    while let Some(&p) = iter.peek() {
        let parameter_item = match extract_single_parameter_from_node(p) {
            Ok(p) => p,
            Err(_) => {
                break;
            }
        };

        match (item_extractor)(parameter_item) {
            Ok(r) => {
                matched_items.push(r);
                iter.next();
            }
            Err(_) => {
                break;
            }
        }
    }
    Ok(matched_items)
}

fn extract_single_parameter_from_node(node: &ParameterNode) -> OpenFinexApiResult<&ParameterItem> {
    match node {
        ParameterNode::SingleParameter(i) => Ok(i),
        _ => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a single parameter, but instead found a nested list of parameters"
        ))),
    }
}

fn extract_nested_parameter_list_from_node(
    node: &ParameterNode,
) -> OpenFinexApiResult<&Vec<ParameterItem>> {
    match node {
        ParameterNode::SingleParameter(_) => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a nested parameter list, but instead found a single parameter"
        ))),
        ParameterNode::ParameterList(l) => Ok(l),
    }
}

fn extract_string_from_item(item: &ParameterItem) -> OpenFinexApiResult<String> {
    match item {
        ParameterItem::String(s) => Ok(s.clone()),
        _ => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a string parameter, but found {}",
            item
        ))),
    }
}

fn extract_json_from_item(item: &ParameterItem) -> OpenFinexApiResult<String> {
    match item {
        ParameterItem::Json(s) => Ok(s.clone()),
        _ => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a JSON string parameter, but found {}",
            item
        ))),
    }
}

fn extract_encoded_string_from_item(item: &ParameterItem) -> OpenFinexApiResult<Vec<u8>> {
    let item_as_string = extract_string_from_item(item)?;
    Ok(item_as_string.encode())
}

fn extract_decimal_from_item(item: &ParameterItem) -> OpenFinexApiResult<PriceAndQuantityType> {
    match item {
        ParameterItem::String(s) => FixedPointNumberConverter::parse_from_string(s),

        // decimals in the response 'should' (according to OpenWare) always be wrapped in a string
        // so if we encounter a 'naked' number, this must be an error
        _ => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a decimal string parameter, but found {}",
            item
        ))),
    }
}

fn extract_integer_from_item(item: &ParameterItem) -> OpenFinexApiResult<ResponseInteger> {
    match item {
        ParameterItem::String(s) => s.parse::<ResponseInteger>().map_err(|_| {
            OpenFinexApiError::ResponseParsingError(format!(
                "Expected an integer parameter, but found {}",
                s
            ))
        }),
        ParameterItem::Number(n) => Ok(*n),
        _ => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected an integer parameter, but found {}",
            item
        ))),
    }
}
