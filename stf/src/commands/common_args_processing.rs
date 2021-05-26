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
    MARKET_ID_BASE_ARG_NAME, MARKET_ID_QUOTE_ARG_NAME, MARKET_TYPE_ARG_NAME, ORDER_SIDE_ARG_NAME,
    ORDER_TYPE_ARG_NAME, PRICE_ARG_NAME, QUANTITY_ARG_NAME, TOKEN_ID_ARG_NAME,
};
use clap::ArgMatches;
use codec::Encode;
use polkadex_sgx_primitives::types::{MarketId, Order, OrderSide, OrderType};
use polkadex_sgx_primitives::{AccountId, AssetId};

pub fn get_order_from_matches(
    matches: &ArgMatches,
    main_account: AccountId,
) -> Result<Order, String> {
    let arg_market_type = matches.value_of(MARKET_TYPE_ARG_NAME).unwrap();

    let arg_order_type = get_order_type_from_str(
        matches
            .value_of(ORDER_TYPE_ARG_NAME)
            .expect(format!("missing {} argument", ORDER_TYPE_ARG_NAME).as_str()),
    );
    if let Err(e) = arg_order_type {
        return Err(e);
    }

    let arg_order_side = get_order_side_from_str(
        matches
            .value_of(ORDER_SIDE_ARG_NAME)
            .expect(format!("missing {} argument", ORDER_SIDE_ARG_NAME).as_str()),
    );
    if let Err(e) = arg_order_side {
        return Err(e);
    }

    let arg_quantity = get_amount_from_matches(matches, QUANTITY_ARG_NAME);
    let arg_price = matches
        .value_of(PRICE_ARG_NAME)
        .map(|v| get_amount_from_str(v));

    let market_id = match get_market_id_from_matches(matches) {
        Ok(m) => m,
        Err(e) => return Err(e),
    };

    let order = Order {
        user_uid: main_account,
        market_id,
        market_type: arg_market_type.encode(),
        order_type: arg_order_type.unwrap(),
        side: arg_order_side.unwrap(),
        quantity: arg_quantity,
        price: arg_price,
    };

    return Ok(order);
}

pub fn get_token_id_from_matches<'a>(matches: &'a ArgMatches<'a>) -> Result<AssetId, String> {
    let token_id_str = matches
        .value_of(TOKEN_ID_ARG_NAME)
        .expect(format!("missing {} argument", TOKEN_ID_ARG_NAME).as_str());

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
    let market_id_base = match get_asset_id_from_str(
        matches
            .value_of(MARKET_ID_BASE_ARG_NAME)
            .expect(format!("missing {} argument", MARKET_ID_BASE_ARG_NAME).as_str()),
    ) {
        Err(e) => return Err(e),
        Ok(b) => b,
    };

    let market_id_quote = match get_asset_id_from_str(
        matches
            .value_of(MARKET_ID_QUOTE_ARG_NAME)
            .expect(format!("missing {} argument", MARKET_ID_QUOTE_ARG_NAME).as_str()),
    ) {
        Err(e) => return Err(e),
        Ok(q) => q,
    };

    Ok(MarketId {
        base: market_id_base,
        quote: market_id_quote,
    })
}

fn get_amount_from_matches(matches: &ArgMatches<'_>, arg_name: &str) -> u128 {
    get_amount_from_str(matches.value_of(arg_name).unwrap())
}

fn get_amount_from_str(arg: &str) -> u128 {
    u128::from_str_radix(arg, 10).expect(&format!("failed to convert {} into an integer", arg))
}

fn get_asset_id_from_str(arg: &str) -> Result<AssetId, String> {
    // Only POLKADEX and DOT supported for now (TODO extend to other asset IDs, using hash arguments)
    match arg.to_ascii_lowercase().as_ref() {
        "polkadex" => Ok(AssetId::POLKADEX),
        "dot" => Ok(AssetId::DOT),
        _ => Err("invalid or unsupported asset ID".to_string()),
    }
}

fn get_order_type_from_str<'a>(arg: &str) -> Result<OrderType, String> {
    match arg.to_ascii_lowercase().as_ref() {
        "limit" => Ok(OrderType::LIMIT),
        "market" => Ok(OrderType::MARKET),
        "postonly" => Ok(OrderType::PostOnly),
        "fillorkill" => Ok(OrderType::FillOrKill),
        _ => Err("invalid order type argument".to_string()),
    }
}

fn get_order_side_from_str<'a>(arg: &str) -> Result<OrderSide, String> {
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
