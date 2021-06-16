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
use crate::openfinex::openfinex_types::{
    OpenFinexDecimal, RequestId, RequestType, ResponseInteger,
};
use crate::openfinex::response_parser::{
    ParameterItem, ParameterNode, ParsedResponse, ResponseMethod,
};
use crate::openfinex::string_serialization::{
    string_to_market_id, string_to_order_side, string_to_order_state, string_to_order_type,
};
use alloc::{string::String, vec::Vec};
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

pub trait OpenFinexResponseObjectMapper {
    fn map_to_response_object(
        &self,
        parsed_response: &ParsedResponse,
    ) -> OpenFinexApiResult<OpenFinexResponse>;
}

pub struct ResponseObjectMapper {}

impl OpenFinexResponseObjectMapper for ResponseObjectMapper {
    fn map_to_response_object(
        &self,
        parsed_response: &ParsedResponse,
    ) -> OpenFinexApiResult<OpenFinexResponse> {
        match &parsed_response.response_method {
            ResponseMethod::Error(s) => Ok(OpenFinexResponse::Error(s.clone())),
            ResponseMethod::FromRequestMethod(rt, ri) => {
                ResponseObjectMapper::map_request_response(rt, ri, &parsed_response.parameters)
            }
            ResponseMethod::TradeEvent => {
                ResponseObjectMapper::map_trade_event(&parsed_response.parameters)
            }
            ResponseMethod::OrderUpdate => {
                ResponseObjectMapper::map_order_update(&parsed_response.parameters)
            }
        }
    }
}

impl ResponseObjectMapper {
    fn map_request_response(
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
            _ => Err(OpenFinexApiError::ResponseParsingError(format!(
                "Unknown or unsupported request type ({}), cannot map to response",
                request_type
            ))),
        }
    }

    fn map_trade_event(parameters: &Vec<ParameterNode>) -> OpenFinexApiResult<OpenFinexResponse> {
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
        let _maker_nickname = get_next_single_item(&mut param_iter, &extract_string_from_item)?;

        let taker_order_id = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;
        let taker_order_uuid =
            get_next_single_item(&mut param_iter, &extract_encoded_string_from_item)?;
        let _taker_uid = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let _taker_nickname = get_next_single_item(&mut param_iter, &extract_string_from_item)?;

        let maker_order_side_str =
            get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let timestamp = get_next_single_item(&mut param_iter, &extract_integer_from_item)?;

        let market_id = string_to_market_id(&market_id_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let maker_side = string_to_order_side(&maker_order_side_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        Ok(OpenFinexResponse::TradeEvent(TradeEvent {
            market_id,
            trade_id,
            price,
            amount,
            funds,
            maker_order_id,
            maker_order_uuid,
            taker_order_id,
            taker_order_uuid,
            maker_side,
            timestamp,
        }))
    }

    fn map_order_update(parameters: &Vec<ParameterNode>) -> OpenFinexApiResult<OpenFinexResponse> {
        let mut param_iter = parameters.iter().peekable();

        let _user_identifier = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
        let _user_nickname = get_next_single_item(&mut param_iter, &extract_string_from_item)?;
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

        let market_id = string_to_market_id(&market_id_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let order_side = string_to_order_side(&order_side_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let order_type = string_to_order_type(&order_type_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let order_state = string_to_order_state(&order_state_str)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        Ok(OpenFinexResponse::OrderUpdate(OrderUpdate {
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

fn extract_encoded_string_from_item(item: &ParameterItem) -> OpenFinexApiResult<Vec<u8>> {
    let item_as_string = extract_string_from_item(item)?;
    Ok(item_as_string.encode())
}

fn extract_decimal_from_item(item: &ParameterItem) -> OpenFinexApiResult<PriceAndQuantityType> {
    match item {
        ParameterItem::String(s) => FixedPointNumberConverter::parse_from_string(s),

        // decimals in the response 'should' (according to OpenWare) always be wrapped in a string
        // so if we encounter a 'naked' number, this must be an error
        ParameterItem::Number(n) => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Expected a decimal string parameter, but found {}",
            n
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
    }
}
