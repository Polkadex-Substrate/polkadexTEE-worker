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

#[cfg(test)]
pub mod utils {

    use crate::{
        commands::common_args::{
            ACCOUNT_ID_ARG_NAME, MARKET_ID_BASE_ARG_NAME, MARKET_ID_QUOTE_ARG_NAME,
            MARKET_TYPE_ARG_NAME, MRENCLAVE_ARG_NAME, ORDER_SIDE_ARG_NAME, ORDER_TYPE_ARG_NAME,
            ORDER_UUID_ARG_NAME, QUANTITY_ARG_NAME, SHARD_ARG_NAME,
        },
        Getter, Index, TrustedGetter, TrustedOperation,
    };
    use clap::{App, Arg, ArgMatches};
    use codec::Encode;

    pub fn create_order_args() -> Vec<String> {
        let market_id_base_arg = format!("--{}=usd", MARKET_ID_BASE_ARG_NAME);
        let market_id_quote_arg = format!("--{}=btc", MARKET_ID_QUOTE_ARG_NAME);
        let market_type_arg = format!("--{}=market_type_002", MARKET_TYPE_ARG_NAME);
        let order_type_arg = format!("--{}=market", ORDER_TYPE_ARG_NAME);
        let order_side_arg = format!("--{}=bid", ORDER_SIDE_ARG_NAME);
        let quantity_arg = format!("--{}=198475", QUANTITY_ARG_NAME);

        vec![
            market_id_base_arg,
            market_id_quote_arg,
            market_type_arg,
            order_type_arg,
            order_side_arg,
            quantity_arg,
        ]
    }

    pub fn create_identifier_args() -> Vec<String> {
        let mrenclave = "HNWNo57rmxEC4jY2EgtGEf1hmkothyMKTEsKMyYWSFZB";
        let mrenclave_arg = format!("--{}={}", MRENCLAVE_ARG_NAME, mrenclave);
        let shard_arg = format!("--{}={}", SHARD_ARG_NAME, mrenclave);

        vec![mrenclave_arg, shard_arg]
    }

    pub fn create_market_id_args() -> Vec<String> {
        let market_id_base_arg = format!("--{}=btc", MARKET_ID_BASE_ARG_NAME);
        let market_id_quote_arg = format!("--{}=usd", MARKET_ID_QUOTE_ARG_NAME);

        vec![market_id_base_arg, market_id_quote_arg]
    }

    pub fn create_order_id_args() -> Vec<String> {
        let order_id_arg = format!("--{}=0l5j0j2lfam", ORDER_UUID_ARG_NAME);

        vec![order_id_arg]
    }

    pub fn create_main_account_args() -> Vec<String> {
        let main_account_arg = format!("--{}=//main_ojwf8a", ACCOUNT_ID_ARG_NAME);

        vec![main_account_arg]
    }

    pub fn add_identifiers_app_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name(MRENCLAVE_ARG_NAME)
                .long(MRENCLAVE_ARG_NAME)
                .takes_value(true)
                .required(true)
                .value_name("base58")
                .help("MRENCLAVE"),
        )
        .arg(
            Arg::with_name(SHARD_ARG_NAME)
                .long(SHARD_ARG_NAME)
                .takes_value(true)
                .required(true)
                .value_name("base58")
                .help("Shard identifier, if only 1 shard, then the same as MRENCLAVE"),
        )
    }

    pub struct PerformOperationMock {}

    impl PerformOperationMock {
        pub fn new() -> Self {
            PerformOperationMock {}
        }

        pub fn perform_operation_mock(
            &self,
            _arg_matches: &ArgMatches<'_>,
            trusted_operation: &TrustedOperation,
        ) -> Option<Vec<u8>> {
            match trusted_operation {
                TrustedOperation::indirect_call(_tcs) => {}
                TrustedOperation::direct_call(_tcs) => {}
                TrustedOperation::get(get) => match get {
                    Getter::public(_) => {}
                    Getter::trusted(tgs) => match &tgs.getter {
                        TrustedGetter::nonce(_account_id) => return Some(Index::encode(&145)),
                        TrustedGetter::free_balance(_) => {}
                        TrustedGetter::reserved_balance(_) => {}
                        TrustedGetter::get_balance(_, _, _) => {}
                    },
                },
            }

            None
        }
    }
}
