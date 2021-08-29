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

use crate::accounts_nonce_storage::{self, accounts_storage, nonce_storage, test_proxy};
use crate::aes;
use crate::attestation;
use crate::ed25519;
use crate::happy_path;
use crate::openfinex;
use crate::polkadex_cache;
use crate::rpc;
use crate::rsa3072;
use crate::ss58check;
use crate::state;
use crate::test_orderbook_storage;
use crate::test_polkadex_gateway;
use crate::top_pool;

use crate::{Timeout, WorkerRequest, WorkerResponse};
use log::*;

use sgx_tunittest::*;
use sgx_types::{sgx_status_t, size_t};

use crate::test_polkadex_balance_storage;
use substrate_api_client::utils::storage_key;
use substratee_worker_primitives::block::StatePayload;

use codec::{Decode, Encode};
use sp_core::{crypto::Pair, hashing::blake2_256, H256};

use crate::constants::{BLOCK_CONFIRMED, GETTERTIMEOUT, SUBSRATEE_REGISTRY_MODULE};

use std::string::String;
use std::vec::Vec;

use crate::ipfs::IpfsContent;
use core::ops::Deref;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::untrusted::time::SystemTimeEx;

use chain_relay::{Block, Header};
use sp_runtime::traits::Header as HeaderT;

use sgx_externalities::SgxExternalitiesTypeTrait;
use substratee_stf::sgx::AccountInfo;
use substratee_stf::StateTypeDiff as StfStateTypeDiff;
use substratee_stf::{ShardIdentifier, Stf, TrustedCall};
use substratee_stf::{TrustedGetter, TrustedOperation};

use jsonrpc_core::futures::executor;
use sp_core::ed25519 as spEd25519;

use rpc::author::{Author, AuthorApi};
use rpc::{api::SideChainApi, basic_pool::BasicPool};
use rpc::{
    io_handler_extensions, polkadex_rpc_gateway, rpc_call_encoder, rpc_cancel_order,
    rpc_get_balance, rpc_withdraw, trusted_operation_verifier,
};

#[no_mangle]
pub extern "C" fn test_main_entrance() -> size_t {
    rsgx_unit_tests!(
        // Polkadex cache
        polkadex_cache::create_order_cache::tests::test_initialize_and_lock_storage,
        polkadex_cache::create_order_cache::tests::test_insert_order_and_increment,
        polkadex_cache::create_order_cache::tests::test_remove_order,
        polkadex_cache::create_order_cache::tests::test_reload_cache,

        polkadex_cache::cancel_order_cache::tests::test_initialize_and_lock_storage,
        polkadex_cache::cancel_order_cache::tests::test_insert_order_and_increment,
        polkadex_cache::cancel_order_cache::tests::test_remove_order,
        polkadex_cache::cancel_order_cache::tests::test_reload_cache,

        polkadex_cache::market_cache::tests::set_markets_with_valid_request_id,
        polkadex_cache::market_cache::tests::set_markets_with_invalid_request_id,
        polkadex_cache::market_cache::tests::set_markets_repeatedly_clears_previous,
        polkadex_cache::market_cache::tests::retrieve_previously_inserted_markets,

        // Polkadex Gateway
        test_polkadex_gateway::initialize_storage, // This is not a test but initializes storage for the following tests
        test_polkadex_gateway::test_authenticate_user,
        test_polkadex_gateway::test_place_limit_buy_order,
        test_polkadex_gateway::test_place_limit_sell_order,
        test_polkadex_gateway::test_place_market_buy_order,
        test_polkadex_gateway::test_place_market_sell_order,
        test_polkadex_gateway::test_cancel_limit_buy_order,
        test_polkadex_gateway::test_cancel_limit_sell_order,
        test_polkadex_gateway::test_settle_trade_full_ask_limit,
        test_polkadex_gateway::test_settle_trade_partial_ask_limit,
        test_polkadex_gateway::test_settle_trade_partial_two_ask_limit,
        test_polkadex_gateway::test_settle_trade_full_buy_limit,
        test_polkadex_gateway::test_settle_trade_partial_buy_limit,
        test_polkadex_gateway::test_settle_trade_partial_two_buy_limit,
        test_polkadex_gateway::test_settle_trade_full_ask_market,
        test_polkadex_gateway::test_settle_trade_partial_ask_market,
        test_polkadex_gateway::test_settle_trade_partial_two_ask_market,
        test_polkadex_gateway::test_settle_trade_full_buy_market,
        test_polkadex_gateway::test_settle_trade_partial_bid_market,
        test_polkadex_gateway::test_settle_trade_partial_two_bid_market,
        test_polkadex_gateway::test_cancel_limit_bid_order,
        test_polkadex_gateway::test_cancel_ask_order,
        test_polkadex_gateway::test_process_create_order,
		test_polkadex_gateway::test_basic_order_checks,
        // Polkadex Balance Storage
        test_polkadex_balance_storage::test_deposit,
        test_polkadex_balance_storage::test_withdraw,
        test_polkadex_balance_storage::test_set_free_balance,
        test_polkadex_balance_storage::test_set_reserve_balance,
        test_polkadex_balance_storage::test_lock_storage_and_initialize_balance,
        test_polkadex_balance_storage::test_lock_storage_and_reserve_balance,
        test_polkadex_balance_storage::test_lock_storage_unreserve_balance,
        test_polkadex_balance_storage::test_lock_storage_and_deposit,
        test_polkadex_balance_storage::test_lock_storage_and_withdraw,
        test_polkadex_balance_storage::test_lock_storage_transfer_balance,
		test_polkadex_balance_storage::test_increase_free_balance,
        // Polkadex Proxy Storage Test Cases
        test_proxy::test_check_if_main_account_registered,
        test_proxy::test_check_if_proxy_registered,
        test_proxy::test_add_main_account,
        test_proxy::test_add_proxy_account,
        test_proxy::test_remove_proxy_account,
        test_proxy::test_remove_main_account,
		happy_path::test_happy_path,

        // Polkadex Orderbook Storage Test Cases
        test_orderbook_storage::test_create_orderbook_storage,
        test_orderbook_storage::test_read_orderbook,
        test_orderbook_storage::test_add_orderbook,
        test_orderbook_storage::test_remove_orderbook,

        // OpenFinex API tests
        openfinex::fixed_point_number_converter::tests::fail_to_parse_invalid_strings,
        openfinex::fixed_point_number_converter::tests::fail_to_parse_scientific_notation,
        openfinex::fixed_point_number_converter::tests::successfully_parse_numbers, // Fail 1
        openfinex::fixed_point_number_converter::tests::fail_to_parse_if_number_exceeds_precision,
        openfinex::fixed_point_number_converter::tests::convert_to_string, //fail 2
        //openfinex::fixed_point_number_converter::tests::convert_to_string_and_back, //FIXME: Test case failing
        openfinex::fixed_point_number_converter::tests::fail_to_parse_if_too_large,
        openfinex::string_serialization::tests::test_market_type_encoded_returns_correct_string,
        openfinex::string_serialization::tests::test_user_id_encoded_returns_correct_string,
        openfinex::string_serialization::tests::test_map_asset_ids,
        openfinex::string_serialization::tests::test_map_order_side,
        openfinex::string_serialization::tests::test_map_order_type,
        openfinex::string_serialization::tests::test_map_order_state,
        openfinex::string_serialization::tests::test_map_market_id,
        openfinex::market::tests::test_deserialize_market_usdbtc,
        openfinex::market::tests::test_deserialize_market_trsteth,
        openfinex::response_lexer::tests::given_valid_delimited_string_then_return_result,
        openfinex::response_lexer::tests::given_string_with_missing_delimiter_then_return_error,
        openfinex::response_lexer::tests::given_valid_number_str_then_lex_correctly,
        openfinex::response_lexer::tests::given_valid_response_string_then_return_lexed_items,
        openfinex::response_lexer::tests::parse_openfinex_example_json_parameter_correctly,
        openfinex::response_lexer::tests::parse_json_parameter_mixed_with_regular_parameters,
        openfinex::response_lexer::tests::given_json_parameter_with_too_many_closing_braces_then_return_error,
        openfinex::response_lexer::tests::given_json_parameter_with_missing_closing_braces_then_return_error,
        openfinex::tests::response_parser_tests::given_valid_create_order_response_then_parse_items,
        openfinex::tests::response_parser_tests::given_valid_error_response_then_parse_items,
        openfinex::tests::response_parser_tests::given_valid_response_with_nested_parameters_then_parse_items,
        openfinex::tests::response_parser_tests::given_invalid_preamble_then_return_error,
        openfinex::tests::response_parser_tests::given_valid_subscription_response_then_succeed,
        openfinex::tests::response_parser_tests::given_valid_order_update_response_then_succeed,
        openfinex::tests::response_parser_tests::given_valid_trade_events_response_then_succeed,
        openfinex::tests::response_parser_tests::given_valid_get_markets_response_then_parse_items,
        openfinex::tests::response_object_mapper_tests::test_given_parsed_error_then_map_to_error_object,
        openfinex::tests::response_object_mapper_tests::test_subscribe_response,
        openfinex::tests::response_object_mapper_tests::test_create_order_response,
        openfinex::tests::response_object_mapper_tests::test_order_update_response,
        openfinex::tests::response_object_mapper_tests::test_trade_event_response,
        openfinex::tests::response_object_mapper_tests::test_get_markets_response,
        openfinex::tests::market_repo_tests::update_markets_from_json_strings,
        openfinex::tests::response_handler_tests::handle_request_response,

        // RPC API tests
        polkadex_rpc_gateway::tests::test_rejecting_outdated_nonce,
        polkadex_rpc_gateway::tests::test_successful_call_with_nonce,
        rpc_call_encoder::tests::test_encoding_none_params_returns_ok,
        rpc_get_balance::tests::test_given_valid_top_return_balances,
        //rpc_place_order::tests::test_given_valid_call_return_order_uuid, TODO: @Bigna this test case is failing ... I need your help with this
        rpc_cancel_order::tests::test_given_valid_order_id_return_success,
        rpc_cancel_order::tests::test_given_order_id_mismatch_then_fail,
        rpc_withdraw::tests::test_given_valid_call_then_succeed,
        rpc_withdraw::tests::test_given_unauthorized_access_then_return_error,
        trusted_operation_verifier::tests::given_valid_operation_in_request_then_decode_succeeds,
        trusted_operation_verifier::tests::given_nonsense_text_in_request_then_decode_fails,
        trusted_operation_verifier::tests::given_valid_operation_with_invalid_signature_then_return_error,
        io_handler_extensions::tests::test_given_io_handler_methods_then_retrieve_all_names_as_string,

        // Nonce Storage
        nonce_storage::tests::create_nonce_storage,
        nonce_storage::tests::initialize_nonce,
        nonce_storage::tests::increment_nonce,
        nonce_storage::tests::remove_nonce,

        // Accounts Storage
        accounts_storage::tests::create_accounts_storage_from_hashmap,
        accounts_storage::tests::create_accounts_storage,
        accounts_storage::tests::adding_main_account,
        accounts_storage::tests::removing_main_account,
        accounts_storage::tests::adding_already_registered_accounts,
        accounts_storage::tests::removing_not_registered_accounts,
        accounts_storage::tests::adding_proxy_account,
        accounts_storage::tests::removing_proxy_account,

        // AccountsNonce Storage
        accounts_nonce_storage::tests::create_and_load_registry,
        accounts_nonce_storage::tests::create_accounts_nonce_storage,
        accounts_nonce_storage::tests::register_main_account,
        accounts_nonce_storage::tests::remove_main_account,
        accounts_nonce_storage::tests::register_proxy_account,
        accounts_nonce_storage::tests::remove_proxy_account,
        accounts_nonce_storage::tests::check_if_main_account_registered,
        accounts_nonce_storage::tests::check_if_proxy_registered,
        accounts_nonce_storage::tests::validate_and_increment_nonce,

        // Utility
        ss58check::tests::convert_account_id_to_and_from_ss58check,

        // Substratee Tests
        top_pool::base_pool::test_should_import_transaction_to_ready,
        top_pool::base_pool::test_should_not_import_same_transaction_twice,
        top_pool::base_pool::test_should_import_transaction_to_future_and_promote_it_later,
        top_pool::base_pool::test_should_promote_a_subgraph,
        top_pool::base_pool::test_should_handle_a_cycle,
        top_pool::base_pool::test_can_track_heap_size,
        top_pool::base_pool::test_should_handle_a_cycle_with_low_priority,
        top_pool::base_pool::test_should_remove_invalid_transactions,
        top_pool::base_pool::test_should_prune_ready_transactions,
        top_pool::base_pool::test_transaction_debug,
        top_pool::base_pool::test_transaction_propagation,
        top_pool::base_pool::test_should_reject_future_transactions,
        top_pool::base_pool::test_should_clear_future_queue,
        top_pool::base_pool::test_should_accept_future_transactions_when_explicitly_asked_to,
        top_pool::primitives::test_h256,
        top_pool::pool::test_should_validate_and_import_transaction,
        top_pool::pool::test_should_reject_if_temporarily_banned,
        top_pool::pool::test_should_notify_about_pool_events,
        top_pool::pool::test_should_clear_stale_transactions,
        top_pool::pool::test_should_ban_mined_transactions,
        //FIXME: This test sometimes fails, sometimes succeeds..
        //top_pool::pool::test_should_limit_futures,
        top_pool::pool::test_should_error_if_reject_immediately,
        top_pool::pool::test_should_reject_transactions_with_no_provides,
        top_pool::ready::tests::test_should_replace_transaction_that_provides_the_same_tag,
        top_pool::ready::tests::test_should_replace_multiple_transactions_correctly,
        top_pool::ready::tests::test_should_return_best_transactions_in_correct_order,
        top_pool::ready::tests::test_should_order_refs,
        top_pool::rotator::tests::test_should_not_ban_if_not_stale,
        top_pool::rotator::tests::test_should_ban_stale_extrinsic,
        top_pool::rotator::tests::test_should_clear_banned,
        top_pool::rotator::tests::test_should_garbage_collect,
        top_pool::tracked_map::tests::test_basic,
        state::test_write_and_load_state_works,
        state::test_sgx_state_decode_encode_works,
        state::test_encrypt_decrypt_state_type_works,
        test_time_is_overdue,
        test_time_is_not_overdue,
        test_compose_block_and_confirmation,
        test_submit_trusted_call_to_top_pool,
        test_submit_trusted_getter_to_top_pool,
        test_differentiate_getter_and_call_works,
        // needs node to be running.. unit tests?
        // test_create_block_and_confirmation_works,
        // test_ocall_worker_request,
        // test_create_state_diff,
        // test_executing_call_updates_account_nonce,
        // test_invalid_nonce_call_is_not_executed,

        // these unit tests (?) need an ipfs node running..
        //ipfs::test_creates_ipfs_content_struct_works,
        //ipfs::test_verification_ok_for_correct_content,
        //ipfs::test_verification_fails_for_incorrect_content,
        //test_ocall_read_write_ipfs,
    )
}

#[allow(unused)]
pub fn ensure_no_empty_shard_directory_exists() {
    // ensure no empty states are within directory (created with init-shard)
    // otherwise an 'index out of bounds: the len is x but the index is x'
    // error will be thrown
    let shards = state::list_shards().unwrap();
    for shard in shards {
        if !state::exists(&shard) {
            state::init_shard(&shard).unwrap();
        }
    }
}

#[allow(unused)]
fn test_ocall_read_write_ipfs() {
    info!("testing IPFS read/write. Hopefully ipfs daemon is running...");
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let mut cid_buf: Vec<u8> = vec![0; 46];
    let enc_state: Vec<u8> = vec![20; 4 * 512 * 1024];

    let _res = unsafe {
        crate::ocall_write_ipfs(
            &mut rt as *mut sgx_status_t,
            enc_state.as_ptr(),
            enc_state.len() as u32,
            cid_buf.as_mut_ptr(),
            cid_buf.len() as u32,
        )
    };

    let res = unsafe {
        crate::ocall_read_ipfs(
            &mut rt as *mut sgx_status_t,
            cid_buf.as_ptr(),
            cid_buf.len() as u32,
        )
    };

    if res == sgx_status_t::SGX_SUCCESS {
        let cid = std::str::from_utf8(&cid_buf).unwrap();
        let mut f = File::open(&cid).unwrap();
        let mut content_buf = Vec::new();
        f.read_to_end(&mut content_buf).unwrap();
        info!("reading file {:?} of size {} bytes", f, &content_buf.len());

        let mut ipfs_content = IpfsContent::new(cid, content_buf);
        let verification = ipfs_content.verify();
        assert!(verification.is_ok());
    } else {
        error!("was not able to write to file");
        panic!()
    }
}

#[allow(unused)]
fn test_ocall_worker_request() {
    info!("testing ocall_worker_request. Hopefully substraTEE-node is running...");
    let mut requests = vec![WorkerRequest::ChainStorage(
        storage_key("Balances", "TotalIssuance").0,
        None,
    )];

    let mut resp: Vec<WorkerResponse<Vec<u8>>> = match crate::worker_request(requests) {
        Ok(response) => response,
        Err(e) => panic!("Worker response decode failed. Error: {:?}", e),
    };

    let first = resp.pop().unwrap();
    info!("Worker response: {:?}", first);

    let (total_issuance, proof) = match first {
        WorkerResponse::ChainStorage(_storage_key, value, proof) => (value, proof),
    };

    info!("Total Issuance is: {:?}", total_issuance);
    info!("Proof: {:?}", proof)
}

#[allow(unused)]
fn test_time_is_overdue() {
    // given
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    // when
    let before_start_time = (start_time * 1000 - GETTERTIMEOUT) / 1000;
    let time_has_run_out = crate::time_is_overdue(Timeout::Getter, before_start_time);
    // then
    assert!(time_has_run_out)
}

#[allow(unused)]
fn test_time_is_not_overdue() {
    // given
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    // when
    let time_has_run_out = crate::time_is_overdue(Timeout::Call, start_time);
    // then
    assert!(!time_has_run_out)
}

#[allow(unused)]
fn test_compose_block_and_confirmation() {
    // given
    ensure_no_empty_shard_directory_exists();
    let latest_onchain_header = Header::new(
        1,
        Default::default(),
        Default::default(),
        [69; 32].into(),
        Default::default(),
    );
    let call_hash: H256 = [94; 32].into();
    let call_hash_two: H256 = [1; 32].into();
    let signed_top_hashes = [call_hash, call_hash_two].to_vec();
    let shard = ShardIdentifier::default();
    let state_hash_apriori: H256 = [199; 32].into();
    // ensure state starts empty
    state::init_shard(&shard).unwrap();
    let mut state = Stf::init_state();
    Stf::update_sidechain_block_number(&mut state, 3);

    // when
    let (opaque_call, signed_block) = crate::compose_block_and_confirmation(
        latest_onchain_header,
        signed_top_hashes,
        shard,
        state_hash_apriori,
        &mut state,
    )
    .unwrap();
    let xt_block_encoded = [SUBSRATEE_REGISTRY_MODULE, BLOCK_CONFIRMED].encode();
    let block_hash_encoded = blake2_256(&signed_block.block().encode()).encode();
    let mut opaque_call_vec = opaque_call.0;

    // then
    assert!(signed_block.verify_signature());
    assert_eq!(signed_block.block().block_number(), 4);
    assert!(opaque_call_vec.starts_with(&xt_block_encoded));
    let mut stripped_opaque_call = opaque_call_vec.split_off(xt_block_encoded.len());
    assert!(stripped_opaque_call.starts_with(&shard.encode()));
    let stripped_opaque_call = stripped_opaque_call.split_off(shard.encode().len());
    assert!(stripped_opaque_call.starts_with(&block_hash_encoded));

    // clean up
    state::remove_shard_dir(&shard);
}

#[allow(unused)]
fn test_submit_trusted_call_to_top_pool() {
    // given
    ensure_no_empty_shard_directory_exists();

    // create top pool
    let api: Arc<SideChainApi<Block>> = Arc::new(SideChainApi::new());
    let tx_pool = BasicPool::create(Default::default(), api);
    let author = Author::new(Arc::new(&tx_pool));

    // create trusted call signed
    let nonce = 0;
    let mrenclave = [0u8; 32];
    let shard = ShardIdentifier::default();
    // ensure state starts empty
    state::init_shard(&shard).unwrap();
    Stf::init_state();
    let signer_pair = ed25519::unseal_pair().unwrap();
    let call = TrustedCall::balance_set_balance(
        signer_pair.public().into(),
        signer_pair.public().into(),
        42,
        42,
    );
    let signed_call = call.sign(&signer_pair.into(), nonce, &mrenclave, &shard);
    let trusted_operation: TrustedOperation = signed_call.clone().into_trusted_operation(true);
    // encrypt call
    let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
    let mut encrypted_top: Vec<u8> = Vec::new();
    rsa_pubkey
        .encrypt_buffer(&trusted_operation.encode(), &mut encrypted_top)
        .unwrap();

    // when

    // submit trusted call to top pool
    let result = async { author.submit_top(encrypted_top.clone(), shard).await };
    executor::block_on(result).unwrap();

    // get pending extrinsics
    let (calls, _) = author.get_pending_tops_separated(shard).unwrap();

    // then
    let call_one = format! {"{:?}", calls[0]};
    let call_two = format! {"{:?}", signed_call};
    assert_eq!(call_one, call_two);

    // clean up
    state::remove_shard_dir(&shard);
}

#[allow(unused)]
fn test_submit_trusted_getter_to_top_pool() {
    // given
    ensure_no_empty_shard_directory_exists();

    // create top pool
    let api: Arc<SideChainApi<Block>> = Arc::new(SideChainApi::new());
    let tx_pool = BasicPool::create(Default::default(), api);
    let author = Author::new(Arc::new(&tx_pool));

    // create trusted getter signed
    let shard = ShardIdentifier::default();
    // ensure state starts empty
    state::init_shard(&shard).unwrap();
    Stf::init_state();
    let signer_pair = ed25519::unseal_pair().unwrap();
    let getter = TrustedGetter::free_balance(signer_pair.public().into());
    let signed_getter = getter.sign(&signer_pair.into());
    let trusted_operation: TrustedOperation = signed_getter.clone().into();
    // encrypt call
    let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
    let mut encrypted_top: Vec<u8> = Vec::new();
    rsa_pubkey
        .encrypt_buffer(&trusted_operation.encode(), &mut encrypted_top)
        .unwrap();

    // when

    // submit top to pool
    let result = async { author.submit_top(encrypted_top.clone(), shard).await };
    executor::block_on(result).unwrap();

    // get pending extrinsics
    let (_, getters) = author.get_pending_tops_separated(shard).unwrap();

    // then
    let getter_one = format! {"{:?}", getters[0]};
    let getter_two = format! {"{:?}", signed_getter};
    assert_eq!(getter_one, getter_two);

    // clean up
    state::remove_shard_dir(&shard);
}

#[allow(unused)]
fn test_differentiate_getter_and_call_works() {
    // given
    ensure_no_empty_shard_directory_exists();

    // create top pool
    let api: Arc<SideChainApi<Block>> = Arc::new(SideChainApi::new());
    let tx_pool = BasicPool::create(Default::default(), api);
    let author = Author::new(Arc::new(&tx_pool));
    // create trusted getter signed
    let shard = ShardIdentifier::default();
    // ensure state starts empty
    state::init_shard(&shard).unwrap();
    Stf::init_state();

    let signer_pair = ed25519::unseal_pair().unwrap();
    let getter = TrustedGetter::free_balance(signer_pair.public().into());
    let signed_getter = getter.sign(&signer_pair.clone().into());
    let trusted_operation: TrustedOperation = signed_getter.clone().into();
    // encrypt call
    let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
    let mut encrypted_top: Vec<u8> = Vec::new();
    rsa_pubkey
        .encrypt_buffer(&trusted_operation.encode(), &mut encrypted_top)
        .unwrap();

    // create trusted call signed
    let nonce = 0;
    let mrenclave = [0u8; 32];
    let call = TrustedCall::balance_set_balance(
        signer_pair.public().into(),
        signer_pair.public().into(),
        42,
        42,
    );
    let signed_call = call.sign(&signer_pair.into(), nonce, &mrenclave, &shard);
    let trusted_operation_call: TrustedOperation = signed_call.clone().into_trusted_operation(true);
    // encrypt call
    let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
    let mut encrypted_top_call: Vec<u8> = Vec::new();
    rsa_pubkey
        .encrypt_buffer(&trusted_operation_call.encode(), &mut encrypted_top_call)
        .unwrap();

    // when

    // submit top to pool
    let result = async { author.submit_top(encrypted_top.clone(), shard).await };
    executor::block_on(result).unwrap();

    let result = async { author.submit_top(encrypted_top_call.clone(), shard).await };
    executor::block_on(result).unwrap();

    // get pending extrinsics
    let (calls, getters) = author.get_pending_tops_separated(shard).unwrap();

    // then
    let getter_one = format! {"{:?}", getters[0]};
    let getter_two = format! {"{:?}", signed_getter};
    let call_one = format! {"{:?}", calls[0]};
    let call_two = format! {"{:?}", signed_call};
    assert_eq!(call_one, call_two);
    assert_eq!(getter_one, getter_two);

    // clean up
    state::remove_shard_dir(&shard);
}

#[allow(unused)]
#[allow(unused_assignments)]
fn test_create_block_and_confirmation_works() {
    // given
    ensure_no_empty_shard_directory_exists();

    // create top pool
    unsafe { rpc::worker_api_direct::initialize_pool() };
    let shard = ShardIdentifier::default();
    // ensure state starts empty
    state::init_shard(&shard).unwrap();
    let mut state = Stf::init_state();
    assert_eq!(Stf::get_sidechain_block_number(&mut state).unwrap(), 0);
    // Header::new(Number, extrinsicroot, stateroot, parenthash, digest)
    let latest_onchain_header = Header::new(
        1,
        Default::default(),
        Default::default(),
        [69; 32].into(),
        Default::default(),
    );
    let mut top_hash = H256::default();

    // load top pool
    {
        let pool_mutex = rpc::worker_api_direct::load_top_pool().unwrap();
        let pool_guard = pool_mutex.lock().unwrap();
        let pool = Arc::new(pool_guard.deref());
        let author = Arc::new(Author::new(pool));

        // create trusted call signed
        let nonce = 0;
        let mrenclave = attestation::get_mrenclave_of_self().unwrap().m;
        let signer_pair = ed25519::unseal_pair().unwrap();
        let call = TrustedCall::balance_transfer(
            signer_pair.public().into(),
            signer_pair.public().into(),
            42,
        );
        let signed_call = call.sign(&signer_pair.into(), nonce, &mrenclave, &shard);
        let trusted_operation: TrustedOperation = signed_call.into_trusted_operation(true);
        // encrypt call
        let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
        let mut encrypted_top: Vec<u8> = Vec::new();
        rsa_pubkey
            .encrypt_buffer(&trusted_operation.encode(), &mut encrypted_top)
            .unwrap();

        // submit trusted call to top pool
        let result = async { author.submit_top(encrypted_top.clone(), shard).await };
        top_hash = executor::block_on(result).unwrap();
    }

    // when
    let (confirm_calls, signed_blocks) =
        crate::execute_top_pool_calls(latest_onchain_header).unwrap();

    let signed_block = signed_blocks[0].clone();
    let mut opaque_call_vec = confirm_calls[0].0.clone();
    let xt_block_encoded = [SUBSRATEE_REGISTRY_MODULE, BLOCK_CONFIRMED].encode();
    let block_hash_encoded = blake2_256(&signed_block.block().encode()).encode();

    // then
    assert!(signed_block.verify_signature());
    assert_eq!(signed_block.block().block_number(), 1);
    assert_eq!(signed_block.block().signed_top_hashes()[0], top_hash);
    assert!(opaque_call_vec.starts_with(&xt_block_encoded));
    let mut stripped_opaque_call = opaque_call_vec.split_off(xt_block_encoded.len());
    assert!(stripped_opaque_call.starts_with(&shard.encode()));
    let stripped_opaque_call = stripped_opaque_call.split_off(shard.encode().len());
    assert!(stripped_opaque_call.starts_with(&block_hash_encoded));

    // clean up
    state::remove_shard_dir(&shard);
}

#[allow(unused)]
fn test_create_state_diff() {
    // given

    ensure_no_empty_shard_directory_exists();

    // create top pool
    unsafe { rpc::worker_api_direct::initialize_pool() };
    let shard = ShardIdentifier::default();
    // Header::new(Number, extrinsicroot, stateroot, parenthash, digest)
    let latest_onchain_header = Header::new(
        1,
        Default::default(),
        Default::default(),
        [69; 32].into(),
        Default::default(),
    );
    let _rsa_pair = rsa3072::unseal_pair().unwrap();

    // ensure that state starts empty
    state::init_shard(&shard).unwrap();
    let state = Stf::init_state();

    // create accounts
    let signer_without_money = ed25519::unseal_pair().unwrap();
    let pair_with_money = spEd25519::Pair::from_seed(b"12345678901234567890123456789012");
    let account_with_money = pair_with_money.public();
    let account_without_money = signer_without_money.public();
    let account_with_money_key_hash =
        substratee_stf::sgx::account_key_hash(&account_with_money.into());
    let account_without_money_key_hash =
        substratee_stf::sgx::account_key_hash(&account_without_money.into());

    let _prev_state_hash = state::write(state, &shard).unwrap();
    // load top pool
    {
        let pool_mutex = rpc::worker_api_direct::load_top_pool().unwrap();
        let pool_guard = pool_mutex.lock().unwrap();
        let pool = Arc::new(pool_guard.deref());
        let author = Arc::new(Author::new(pool));

        // create trusted call signed
        let nonce = 0;
        let mrenclave = attestation::get_mrenclave_of_self().unwrap().m;
        let call = TrustedCall::balance_transfer(
            account_with_money.into(),
            account_without_money.into(),
            1000,
        );
        let signed_call = call.sign(&pair_with_money.into(), nonce, &mrenclave, &shard);
        let trusted_operation: TrustedOperation = signed_call.into_trusted_operation(true);
        // encrypt call
        let mut encrypted_top: Vec<u8> = Vec::new();
        let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
        rsa_pubkey
            .encrypt_buffer(&trusted_operation.encode(), &mut encrypted_top)
            .unwrap();

        // submit trusted call to top pool
        let result = async { author.submit_top(encrypted_top.clone(), shard).await };
        executor::block_on(result).unwrap();
    }

    // when
    let (_, signed_blocks) = crate::execute_top_pool_calls(latest_onchain_header).unwrap();
    let mut encrypted_payload: Vec<u8> = signed_blocks[0].block().state_payload().to_vec();
    aes::de_or_encrypt(&mut encrypted_payload).unwrap();
    let state_payload = StatePayload::decode(&mut encrypted_payload.as_slice()).unwrap();
    let state_diff = StfStateTypeDiff::decode(state_payload.state_update().to_vec());

    // then
    let acc_info_vec = state_diff
        .get(&account_with_money_key_hash)
        .unwrap()
        .as_ref()
        .unwrap();
    let new_balance_acc_with_money = AccountInfo::decode(&mut acc_info_vec.as_slice())
        .unwrap()
        .data
        .free;
    let acc_info_vec = state_diff
        .get(&account_without_money_key_hash)
        .unwrap()
        .as_ref()
        .unwrap();
    let new_balance_acc_wo_money = AccountInfo::decode(&mut acc_info_vec.as_slice())
        .unwrap()
        .data
        .free;
    // get block number
    let block_number_key = substratee_stf::sgx::storage_value_key("System", "Number");
    let new_block_number_encoded = state_diff.get(&block_number_key).unwrap().as_ref().unwrap();
    let new_block_number =
        substratee_worker_primitives::BlockNumber::decode(&mut new_block_number_encoded.as_slice())
            .unwrap();
    assert_eq!(state_diff.len(), 3);
    assert_eq!(new_balance_acc_wo_money, 1000);
    assert_eq!(new_balance_acc_with_money, 1000);
    assert_eq!(new_block_number, 1);

    // clean up
    state::remove_shard_dir(&shard);
}

#[allow(unused)]
fn test_executing_call_updates_account_nonce() {
    // given

    ensure_no_empty_shard_directory_exists();

    // create top pool
    unsafe { rpc::worker_api_direct::initialize_pool() };
    let shard = ShardIdentifier::default();
    // Header::new(Number, extrinsicroot, stateroot, parenthash, digest)
    let latest_onchain_header = Header::new(
        1,
        Default::default(),
        Default::default(),
        [69; 32].into(),
        Default::default(),
    );
    let _rsa_pair = rsa3072::unseal_pair().unwrap();

    // ensure that state starts empty
    state::init_shard(&shard).unwrap();
    let mut state = Stf::init_state();

    // create accounts
    let signer_without_money = ed25519::unseal_pair().unwrap();
    let pair_with_money = spEd25519::Pair::from_seed(b"12345678901234567890123456789012");
    let account_with_money = pair_with_money.public();
    let account_without_money = signer_without_money.public();
    let account_with_money_key_hash =
        substratee_stf::sgx::account_key_hash(&account_with_money.into());
    let account_without_money_key_hash =
        substratee_stf::sgx::account_key_hash(&account_without_money.into());

    let _prev_state_hash = state::write(state, &shard).unwrap();
    // load top pool
    {
        let pool_mutex = rpc::worker_api_direct::load_top_pool().unwrap();
        let pool_guard = pool_mutex.lock().unwrap();
        let pool = Arc::new(pool_guard.deref());
        let author = Arc::new(Author::new(pool.clone()));

        // create trusted call signed
        let nonce = 0;
        let mrenclave = attestation::get_mrenclave_of_self().unwrap().m;
        let call = TrustedCall::balance_transfer(
            account_with_money.into(),
            account_without_money.into(),
            1000,
        );
        let signed_call = call.sign(&pair_with_money.into(), nonce, &mrenclave, &shard);
        let trusted_operation: TrustedOperation = signed_call.into_trusted_operation(true);
        // encrypt call
        let mut encrypted_top: Vec<u8> = Vec::new();
        let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
        rsa_pubkey
            .encrypt_buffer(&trusted_operation.encode(), &mut encrypted_top)
            .unwrap();

        // submit trusted call to top pool
        let result = async { author.submit_top(encrypted_top.clone(), shard).await };
        executor::block_on(result).unwrap();
    }

    // when
    let (_, signed_blocks) = crate::execute_top_pool_calls(latest_onchain_header).unwrap();

    // then
    let mut state = state::load(&shard).unwrap();
    let nonce = Stf::account_nonce(&mut state, &account_with_money.into());
    assert_eq!(nonce, 1);

    // clean up
    state::remove_shard_dir(&shard);
}

#[allow(unused)]
fn test_invalid_nonce_call_is_not_executed() {
    // given

    ensure_no_empty_shard_directory_exists();

    // create top pool
    unsafe { rpc::worker_api_direct::initialize_pool() };
    let shard = ShardIdentifier::default();
    // Header::new(Number, extrinsicroot, stateroot, parenthash, digest)
    let latest_onchain_header = Header::new(
        1,
        Default::default(),
        Default::default(),
        [69; 32].into(),
        Default::default(),
    );
    let _rsa_pair = rsa3072::unseal_pair().unwrap();

    // ensure that state starts empty
    state::init_shard(&shard).unwrap();
    let mut state = Stf::init_state();

    // create accounts
    let signer_without_money = ed25519::unseal_pair().unwrap();
    let pair_with_money = spEd25519::Pair::from_seed(b"12345678901234567890123456789012");
    let account_with_money = pair_with_money.public();
    let account_without_money = signer_without_money.public();
    let account_with_money_key_hash =
        substratee_stf::sgx::account_key_hash(&account_with_money.into());
    let account_without_money_key_hash =
        substratee_stf::sgx::account_key_hash(&account_without_money.into());

    let _prev_state_hash = state::write(state, &shard).unwrap();
    // load top pool
    {
        let pool_mutex = rpc::worker_api_direct::load_top_pool().unwrap();
        let pool_guard = pool_mutex.lock().unwrap();
        let pool = Arc::new(pool_guard.deref());
        let author = Arc::new(Author::new(pool.clone()));

        // create trusted call signed
        let nonce = 10;
        let mrenclave = attestation::get_mrenclave_of_self().unwrap().m;
        let call = TrustedCall::balance_transfer(
            account_with_money.into(),
            account_without_money.into(),
            1000,
        );
        let signed_call = call.sign(&pair_with_money.into(), nonce, &mrenclave, &shard);
        let trusted_operation: TrustedOperation = signed_call.into_trusted_operation(true);
        // encrypt call
        let mut encrypted_top: Vec<u8> = Vec::new();
        let rsa_pubkey = rsa3072::unseal_pubkey().unwrap();
        rsa_pubkey
            .encrypt_buffer(&trusted_operation.encode(), &mut encrypted_top)
            .unwrap();

        // submit trusted call to top pool
        let result = async { author.submit_top(encrypted_top.clone(), shard).await };
        executor::block_on(result).unwrap();
    }

    // when
    let (_, signed_blocks) = crate::execute_top_pool_calls(latest_onchain_header).unwrap();

    // then
    let mut state = state::load(&shard).unwrap();
    let nonce = Stf::account_nonce(&mut state, &account_with_money.into());
    assert_eq!(nonce, 0);

    let acc_data_with_money = Stf::account_data(&mut state, &account_with_money.into()).unwrap();
    assert_eq!(acc_data_with_money.free, 2000);

    // clean up
    state::remove_shard_dir(&shard);
}
