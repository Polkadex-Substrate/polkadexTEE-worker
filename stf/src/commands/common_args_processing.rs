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
use crate::commands::common_args::{
    MARKET_ID_BASE_ARG_NAME, MARKET_ID_QUOTE_ARG_NAME, MARKET_TYPE_ARG_NAME, ORDER_SIDE_ARG_NAME,
    ORDER_TYPE_ARG_NAME, ORDER_UUID_ARG_NAME, PRICE_ARG_NAME, QUANTITY_ARG_NAME, TOKEN_ID_ARG_NAME,
};
use clap::ArgMatches;
use polkadex_sgx_primitives::types::{CancelOrder, MarketId, Order, OrderSide, OrderType};
use polkadex_sgx_primitives::{AccountId, AssetId};

pub fn get_order_from_matches(
    matches: &ArgMatches,
    main_account: AccountId,
) -> Result<Order, String> {
    let arg_market_type = matches.value_of(MARKET_TYPE_ARG_NAME).unwrap();

    let arg_order_type = get_order_type_from_str(
        matches
            .value_of(ORDER_TYPE_ARG_NAME)
            .unwrap_or_else(|| panic!("missing {} argument", ORDER_TYPE_ARG_NAME)),
    )?;

    let arg_order_side = get_order_side_from_str(
        matches
            .value_of(ORDER_SIDE_ARG_NAME)
            .unwrap_or_else(|| panic!("missing {} argument", ORDER_SIDE_ARG_NAME)),
    )?;

    let arg_quantity = get_amount_from_matches(matches, QUANTITY_ARG_NAME);
    let arg_price = matches
        .value_of(PRICE_ARG_NAME)
        .map(|v| get_amount_from_str(v));

    let market_id = get_market_id_from_matches(matches)?;

    let order = Order {
        user_uid: main_account,
        market_id,
        market_type: arg_market_type.to_string().into_bytes(), // use utf-8 encoding
        order_type: arg_order_type,
        side: arg_order_side,
        quantity: arg_quantity,
        price: arg_price,
    };

    Ok(order)
}

pub fn get_cancel_order_from_matches(
    matches: &ArgMatches,
    main_account: AccountId,
) -> Result<CancelOrder, String> {
    let order_id = matches
        .value_of(ORDER_UUID_ARG_NAME)
        .unwrap_or_else(|| panic!("missing {} argument", ORDER_UUID_ARG_NAME))
        .as_bytes()
        .to_vec();

    let market_id = get_market_id_from_matches(matches)?;

    let order = CancelOrder {
        user_uid: main_account,
        market_id,
        order_id,
    };

    Ok(order)
}

pub fn get_token_id_from_matches<'a>(matches: &'a ArgMatches<'a>) -> Result<AssetId, String> {
    let token_id_str = matches
        .value_of(TOKEN_ID_ARG_NAME)
        .unwrap_or_else(|| panic!("missing {} argument", TOKEN_ID_ARG_NAME));

    get_asset_id_from_str(token_id_str)
}

pub fn get_quantity_from_matches(matches: &ArgMatches) -> Result<u128, String> {
    let quantity_option = matches.value_of(QUANTITY_ARG_NAME);

    match quantity_option {
        Some(quantity_str) => Ok(get_amount_from_str(quantity_str)),
        None => Err(format!("missing {} argument", QUANTITY_ARG_NAME)),
    }
}

fn get_market_id_from_matches<'a>(matches: &'a ArgMatches<'a>) -> Result<MarketId, String> {
    let market_id_base = get_asset_id_from_str(
        matches
            .value_of(MARKET_ID_BASE_ARG_NAME)
            .unwrap_or_else(|| panic!("missing {} argument", MARKET_ID_BASE_ARG_NAME)),
    )?;

    let market_id_quote = get_asset_id_from_str(
        matches
            .value_of(MARKET_ID_QUOTE_ARG_NAME)
            .unwrap_or_else(|| panic!("missing {} argument", MARKET_ID_QUOTE_ARG_NAME)),
    )?;

    Ok(MarketId {
        base: market_id_base,
        quote: market_id_quote,
    })
}

fn get_amount_from_matches(matches: &ArgMatches<'_>, arg_name: &str) -> u128 {
    get_amount_from_str(matches.value_of(arg_name).unwrap())
}

fn get_amount_from_str(arg: &str) -> u128 {
    arg.parse::<u128>()
        .unwrap_or_else(|_| panic!("failed to convert {} into an integer", arg))
}

fn get_asset_id_from_str(arg: &str) -> Result<AssetId, String> {
    // Only POLKADEX and DOT supported for now (TODO extend to other asset IDs, using hash arguments)
    match arg.to_ascii_lowercase().as_ref() {
        "polkadex" => Ok(AssetId::POLKADEX),
        "dot" => Ok(AssetId::DOT),
        "btc" => Ok(AssetId::BTC),
        "usd" => Ok(AssetId::USD),
        _ => Err("invalid or unsupported asset ID".to_string()),
    }
}

fn get_order_type_from_str(arg: &str) -> Result<OrderType, String> {
    match arg.to_ascii_lowercase().as_ref() {
        "limit" => Ok(OrderType::LIMIT),
        "market" => Ok(OrderType::MARKET),
        "postonly" => Ok(OrderType::PostOnly),
        "fillorkill" => Ok(OrderType::FillOrKill),
        _ => Err("invalid order type argument".to_string()),
    }
}

fn get_order_side_from_str(arg: &str) -> Result<OrderSide, String> {
    match arg.to_ascii_lowercase().as_ref() {
        "bid" => Ok(OrderSide::BID),
        "ask" => Ok(OrderSide::ASK),
        _ => Err("invalid order side argument".to_string()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::commands::common_args::add_order_args;
    use crate::commands::test_utils::utils::create_order_args;
    use clap::{App, AppSettings};
    use sp_application_crypto::sr25519;
    use sp_core::{sr25519 as sr25519_core, Pair};

    #[test]
    pub fn given_correct_args_then_map_to_order() {
        let order_args = create_order_args();
        let matches = create_test_app().get_matches_from(order_args);

        let main_account_key_pair = sr25519::AppPair::from_string("//test-account", None).unwrap();
        let main_account: AccountId =
            sr25519_core::Public::from(main_account_key_pair.public()).into();

        let order_mapping_result = get_order_from_matches(&matches, main_account);

        assert!(order_mapping_result.is_ok());

        let order = order_mapping_result.unwrap();
        assert_eq!(order.order_type, OrderType::MARKET);
        assert_eq!(order.side, OrderSide::BID);
        assert_eq!(order.quantity, 198475);
        assert_eq!(order.market_id.base, AssetId::POLKADEX);
        assert_eq!(order.market_id.quote, AssetId::DOT);
    }

    fn create_test_app<'a, 'b>() -> App<'a, 'b> {
        let test_app = App::new("test_account_details").setting(AppSettings::NoBinaryName);
        add_order_args(test_app)
    }
}
