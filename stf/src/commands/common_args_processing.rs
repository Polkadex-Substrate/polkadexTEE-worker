/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/

use crate::commands::common_args::{
    MARKET_ID_ARG_NAME, MARKET_TYPE_ARG_NAME, ORDER_SIDE_ARG_NAME, ORDER_TYPE_ARG_NAME,
    PRICE_ARG_NAME, QUANTITY_ARG_NAME,
};
use clap::ArgMatches;
use codec::Encode;
use polkadex_sgx_primitives::types::{Order, OrderSide, OrderType};

pub fn get_order_from_matches<'a>(
    account: &str,
    matches: &ArgMatches<'a>,
) -> Result<Order, &'a str> {
    let arg_market_id = matches.value_of(MARKET_ID_ARG_NAME).unwrap();
    let arg_market_type = matches.value_of(MARKET_TYPE_ARG_NAME).unwrap();

    let arg_order_type = get_order_type_from_str(matches.value_of(ORDER_TYPE_ARG_NAME).unwrap());
    if let Err(e) = arg_order_type {
        return Err(e);
    }

    let arg_order_side = get_order_side_from_str(matches.value_of(ORDER_SIDE_ARG_NAME).unwrap());
    if let Err(e) = arg_order_side {
        return Err(e);
    }

    let arg_quantity = get_amount_from_matches(matches, QUANTITY_ARG_NAME);
    let arg_price = matches
        .value_of(PRICE_ARG_NAME)
        .map(|v| get_amount_from_str(v));

    let order = Order {
        user_uid: account.encode(),
        market_id: arg_market_id.encode(),
        market_type: arg_market_type.encode(),
        order_type: arg_order_type.unwrap(),
        side: arg_order_side.unwrap(),
        quantity: arg_quantity,
        price: arg_price,
    };

    return Ok(order);
}

fn get_amount_from_matches(matches: &ArgMatches<'_>, arg_name: &str) -> u128 {
    get_amount_from_str(matches.value_of(arg_name).unwrap())
}

fn get_amount_from_str(arg: &str) -> u128 {
    u128::from_str_radix(arg, 10).expect(&format!("failed to convert {} into an integer", arg))
}

fn get_order_type_from_str<'a>(arg: &str) -> Result<OrderType, &'a str> {
    match arg.to_ascii_lowercase().as_ref() {
        "limit" => Ok(OrderType::LIMIT),
        "market" => Ok(OrderType::MARKET),
        "postonly" => Ok(OrderType::PostOnly),
        "fillorkill" => Ok(OrderType::FillOrKill),
        _ => Err("invalid order type argument"),
    }
}

fn get_order_side_from_str<'a>(arg: &str) -> Result<OrderSide, &'a str> {
    match arg.to_ascii_lowercase().as_ref() {
        "bid" => Ok(OrderSide::BID),
        "ask" => Ok(OrderSide::ASK),
        _ => Err("invalid order side argument"),
    }
}
