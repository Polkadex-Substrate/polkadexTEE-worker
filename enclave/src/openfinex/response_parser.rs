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
//use crate::openfinex::openfinex_api::OpenFinexApiError::ResponseParsingError;
use crate::openfinex::openfinex_api::{OpenFinexApiError, OpenFinexApiResult};
use crate::openfinex::openfinex_types::{Preamble, RequestId, RequestType, ResponseInteger};
use crate::openfinex::response_lexer::{LexItem, ResponseLexer};
use alloc::{
    fmt::Result as FormatResult, string::String, string::ToString,
    vec::Vec,
};
use core::iter::Peekable;

const RESPONSE_TO_REQUEST_PREAMBLE: Preamble = 2;
const RESPONSE_TO_EVENTS: Preamble = 5;

/// The parsed response object from OpenFinex
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedResponse {
    pub response_preamble: Preamble,
    pub response_method: ResponseMethod,
    pub parameters: Vec<ParameterNode>,
}

/// The request method in the response, may also be set to 'error'
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseMethod {
    FromRequestMethod(RequestType, RequestId),
    OrderUpdate,
    TradeEvent,
    Error(String),
}

/// a parsed parameter node, which itself can be a list of parameters
/// (only 1-level deep recursion though)
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterNode {
    SingleParameter(ParameterItem),
    ParameterList(Vec<ParameterItem>),
}

/// a single parameter item, can either be a string or number
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterItem {
    String(String),
    Number(ResponseInteger),
}

impl alloc::fmt::Display for ParameterItem {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> FormatResult {
        write!(f, "{:?}", self)
    }
}

/// Trait for a generalized response parser
pub trait ResponseParser {
    fn parse_response_string(&self, response: String) -> OpenFinexApiResult<ParsedResponse>;
}

/// Parses an OpenFinex response String
///
pub struct TcpResponseParser {}

impl TcpResponseParser {
    const RESPONSE_ERROR_STR: &'static str = "error";
}

impl ResponseParser for TcpResponseParser {
    fn parse_response_string(&self, response: String) -> OpenFinexApiResult<ParsedResponse> {
        let lexer = ResponseLexer {};
        let lex_items = lexer
            .lex(&response)
            .map_err(|e| OpenFinexApiError::ResponseParsingError(e))?;

        let mut lex_it = lex_items.iter().peekable();

        consume_token(&mut lex_it, &token_opening_parenthesis)?;

        let preamble = consume_token(&mut lex_it, &token_number)?;

        let request_id = parse_request_id(&mut lex_it, preamble)?;

        let response_method_str = consume_token(&mut lex_it, &token_string)?;

        let parameters = parse_parameters(&mut lex_it)?;

        consume_token(&mut lex_it, &token_closing_parenthesis)?;

        let response_method =
            parse_response_method(&response_method_str, &parameters, &request_id)?;

        Ok(ParsedResponse {
            parameters,
            response_method,
            response_preamble: preamble,
        })
    }
}

fn parse_request_id<'a, T: Iterator<Item = &'a LexItem>>(
    iter: &mut Peekable<T>,
    preamble: Preamble,
) -> OpenFinexApiResult<Option<RequestId>> {
    match preamble {
        RESPONSE_TO_REQUEST_PREAMBLE => consume_token(iter, &token_number).map(|r| Some(r)),
        RESPONSE_TO_EVENTS => Ok(None),
        _ => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Unknown response preamble: {}",
            preamble
        ))),
    }
}

fn parse_response_method(
    input: &String,
    parameters: &Vec<ParameterNode>,
    request_id: &Option<RequestId>,
) -> OpenFinexApiResult<ResponseMethod> {
    match input.as_str() {
        // error
        TcpResponseParser::RESPONSE_ERROR_STR => {
            let error_message = retrieve_error_message_from_parameters(parameters);
            Ok(ResponseMethod::Error(error_message))
        }

        _ => {
            match request_id {
                // we have a request ID, so the response method must map to a request method
                Some(rid) => RequestType::from_request_string(input)
                    .map(|rt| ResponseMethod::FromRequestMethod(rt, *rid)),
                // no request ID, so it's not a request, but an update/event
                None => map_response_method_update(input),
            }
        }
    }
}

fn map_response_method_update(input: &String) -> OpenFinexApiResult<ResponseMethod> {
    match input.as_str() {
        "ou" => Ok(ResponseMethod::OrderUpdate),
        "tr" => Ok(ResponseMethod::TradeEvent),
        _ => Err(OpenFinexApiError::ResponseParsingError(format!(
            "Unknown response method {}",
            input
        ))),
    }
}

fn retrieve_error_message_from_parameters(parameters: &Vec<ParameterNode>) -> String {
    // the first parameter, which should be a string, is the error message
    match parameters.first() {
        Some(p) => match p {
            ParameterNode::SingleParameter(i) => match i {
                ParameterItem::String(s) => s.clone(),
                _ => format!("//failed to get error description from response//"),
            },
            _ => format!("//failed to get error description from response//"),
        },
        None => format!("//no error description was found in the response//"),
    }
}

fn parse_parameters<'a, T: Iterator<Item = &'a LexItem>>(
    iter: &mut Peekable<T>,
) -> OpenFinexApiResult<Vec<ParameterNode>> {
    consume_token(iter, &token_opening_parenthesis)?;

    let mut parameters = Vec::<ParameterNode>::new();

    while let Some(t) = peek_token(iter, &token_parameter) {
        parameters.push(ParameterNode::SingleParameter(t));
        iter.next();
    }

    // check for nested parameters list
    if let Some(_) = peek_token(iter, &token_opening_parenthesis) {
        let nested_parameters = parse_parameters(iter)?; // recursive call

        // we flatten all nested parameters because we only support recursion depth=1
        // otherwise we'd need a tree structure to hold the data, which is not a commonly
        // available data structure in rust afaik
        let flattened_parameters = nested_parameters
            .iter()
            .map(|pp| match pp {
                ParameterNode::SingleParameter(t) => vec![t.clone()],
                ParameterNode::ParameterList(l) => l.clone(),
            })
            .flatten()
            .collect::<Vec<ParameterItem>>();

        parameters.push(ParameterNode::ParameterList(flattened_parameters));
    }

    // // continue parameter list
    // while let Some(t) = try_token(iter, &token_string) {
    //     parameters.push(ParsedParameter::SingleParameter(t));
    //     iter.next();
    // }

    consume_token(iter, &token_closing_parenthesis)?;

    Ok(parameters)
}

fn consume_token<'a, T: Iterator<Item = &'a LexItem>, R>(
    iter: &mut Peekable<T>,
    parse_fn: &dyn Fn(&LexItem) -> OpenFinexApiResult<R>,
) -> OpenFinexApiResult<R> {
    match iter.next() {
        Some(l) => (parse_fn)(l),
        None => Err(unexpected_end_error()),
    }
}

fn peek_token<'a, T: Iterator<Item = &'a LexItem>, R>(
    iter: &mut Peekable<T>,
    parse_fn: &dyn Fn(&LexItem) -> OpenFinexApiResult<R>,
) -> Option<R> {
    match iter.peek() {
        Some(l) => match (parse_fn)(*l) {
            Ok(t) => Some(t),
            Err(_) => None,
        },
        None => None,
    }
}

fn token_parameter(lex_item: &LexItem) -> OpenFinexApiResult<ParameterItem> {
    if let Ok(s) = token_string(lex_item) {
        return Ok(ParameterItem::String(s));
    }

    if let Ok(n) = token_number(lex_item) {
        return Ok(ParameterItem::Number(n));
    }

    let expected_string_item = LexItem::String("any".to_string());
    let expected_number_item = LexItem::Number(1);

    Err(unexpected_token_error_for_multiple(
        &vec![&expected_string_item, &expected_number_item],
        lex_item,
    ))
}

fn token_string(lex_item: &LexItem) -> OpenFinexApiResult<String> {
    let expected_item = LexItem::String("any".to_string());
    match lex_item {
        LexItem::String(s) => Ok(s.clone()),
        _ => Err(unexpected_token_error(&expected_item, lex_item)),
    }
}

fn token_number(lex_item: &LexItem) -> OpenFinexApiResult<ResponseInteger> {
    let expected_item = LexItem::Number(1);
    match lex_item {
        LexItem::Number(i) => Ok(*i),
        _ => Err(unexpected_token_error(&expected_item, lex_item)),
    }
}

fn token_opening_parenthesis(lex_item: &LexItem) -> OpenFinexApiResult<()> {
    let expected_item = LexItem::Paren('[');
    match lex_item {
        LexItem::Paren('[') => Ok(()),
        _ => Err(unexpected_token_error(&expected_item, lex_item)),
    }
}

fn token_closing_parenthesis(lex_item: &LexItem) -> OpenFinexApiResult<()> {
    let expected_item = LexItem::Paren(']');
    match lex_item {
        LexItem::Paren(']') => Ok(()),
        _ => Err(unexpected_token_error(&expected_item, lex_item)),
    }
}

fn unexpected_token_error_for_multiple(
    expected: &Vec<&LexItem>,
    actual: &LexItem,
) -> OpenFinexApiError {
    OpenFinexApiError::ResponseParsingError(format!(
        "Unexpected token {}, expected '{:?}'",
        actual, expected
    ))
}

fn unexpected_token_error(expected: &LexItem, actual: &LexItem) -> OpenFinexApiError {
    unexpected_token_error_for_multiple(&vec![expected], actual)
}

fn unexpected_end_error() -> OpenFinexApiError {
    OpenFinexApiError::ResponseParsingError(format!("Unexpected end of tokens, expected any"))
}
