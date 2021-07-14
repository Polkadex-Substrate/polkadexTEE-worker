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
use crate::openfinex::openfinex_types::RequestType;
use crate::openfinex::response_parser::{
    ParameterNode, ResponseMethod, ResponseParser, TcpResponseParser,
};
use alloc::{string::String, string::ToString};

pub fn given_valid_create_order_response_then_parse_items() {
    let response_string = "[2,42,\"admin_create_order\",[\"1245-2345-6798-123123\"]]".to_string();

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string).unwrap();

    assert_eq!(parsed_response.response_preamble, 2);
    assert_eq!(
        parsed_response.response_method,
        ResponseMethod::FromRequestMethod(RequestType::CreateOrder, 42)
    );
    assert_eq!(parsed_response.parameters.len(), 1);
}

pub fn given_valid_get_markets_response_then_parse_items() {
    let response_string = (r#"[2,1,"get_markets",[{"id":"btcusd","name":"BTC/USD","base_unit":"btc","quote_unit":"usd","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":100,"filters":[]},{"id":"trsteth","name":"TRST/ETH","base_unit":"trst","quote_unit":"eth","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":105,"filters":[]}]]"#).to_string();

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string).unwrap();

    assert_eq!(parsed_response.response_preamble, 2);
    assert_eq!(
        parsed_response.response_method,
        ResponseMethod::FromRequestMethod(RequestType::GetMarkets, 1)
    );
    assert_eq!(parsed_response.parameters.len(), 2);
}

pub fn given_valid_error_response_then_parse_items() {
    let error_description = String::from("Message describing the error");
    let response_string = format!("[2,42,\"error\",[\"{}\"]]", error_description);

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string).unwrap();

    assert_eq!(parsed_response.response_preamble, 2);
    assert_eq!(
        parsed_response.response_method,
        ResponseMethod::Error(error_description)
    );
    assert_eq!(parsed_response.parameters.len(), 1);
}

pub fn given_invalid_preamble_then_return_error() {
    let invalid_preamble = 1847;
    let response_string = format!(
        "[{},42,\"admin_create_order\",[\"1245-2345-6798-123123\"]]",
        invalid_preamble
    );

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string);

    assert!(parsed_response.is_err());
}

pub fn given_valid_response_with_nested_parameters_then_parse_items() {
    let response_string = "[2,987132451,\"admin_cancel_order\",[\"btcusd\", [\"1245-2345-6798-123123\", 34, \"1245-2345-
            6798-123124\"]]]".to_string();

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string).unwrap();

    assert_eq!(parsed_response.response_preamble, 2);
    assert_eq!(
        parsed_response.response_method,
        ResponseMethod::FromRequestMethod(RequestType::CancelOrder, 987132451)
    );
    assert_eq!(parsed_response.parameters.len(), 2);

    match parsed_response
        .parameters
        .last()
        .expect("parameters to have 2 elements")
    {
        ParameterNode::ParameterList(l) => {
            assert_eq!(l.len(), 3);
        }
        _ => assert!(
            false,
            "Wrong type of parsed parameter, expected list, found single element!"
        ),
    };
}

pub fn given_valid_subscription_response_then_succeed() {
    let response_string =
        "[2,51,\"subscribe\",[\"admin\",[\"events.order\",\"events.trade\"]]]".to_string();

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string).unwrap();

    assert_eq!(
        parsed_response.response_method,
        ResponseMethod::FromRequestMethod(RequestType::Subscribe, 51)
    );
}

pub fn given_valid_order_update_response_then_succeed() {
    let response_string = "[5,\"ou\",[\"ABC000001\", \"0x1234567890123456789\", \"btcusd\", 2, \"7acbbc84-939d-11eaa827-
            1831bf9834b0\", \"buy\", \"d\", \"l\", \"1\", \"1\", \"0\", \"1\", \"1\", 1, 1589211516]]".to_string();

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string).unwrap();

    assert_eq!(parsed_response.response_method, ResponseMethod::OrderUpdate);
}

pub fn given_valid_trade_events_response_then_succeed() {
    let response_string = "[5,\"tr\",[\"btcusd\", 1, \"1000\", \"2\", \"2000\", 2, \"55d78eee-939e-11ea-945f-
            1831bf9834b0\", \"A00001\", \"0x000001\", 3, \"55d78eee-939e-11ea-945f-1831bf9834as\",\"A00002\", \"0x000002\", \"buy\", 1589211884]]".to_string();

    let parser = TcpResponseParser {};
    let parsed_response = parser.parse_response_string(response_string).unwrap();

    assert_eq!(parsed_response.response_method, ResponseMethod::TradeEvent);
}
