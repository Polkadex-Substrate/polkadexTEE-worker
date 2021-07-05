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

#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(core_intrinsics)]
#![feature(derive_eq)]
#![crate_name = "substratee_worker_enclave"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![allow(clippy::missing_safety_doc)]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use crate::constants::{
    CALL_WORKER, OCEX_DEPOSIT, OCEX_MODULE, OCEX_RELEASE, OCEX_WITHDRAW, SHIELD_FUNDS,
};
use crate::polkadex_nonce_storage::{lock_storage_and_get_nonce, lock_storage_and_increment_nonce};
use crate::utils::UnwrapOrSgxErrorUnexpected;
use base58::ToBase58;
use chain_relay::{
    storage_proof::{StorageProof, StorageProofChecker},
    Block, Header, LightValidation,
};
use codec::{Decode, Encode};
use constants::{
    BLOCK_CONFIRMED, CALLTIMEOUT, CALL_CONFIRMED, GETTERTIMEOUT, OCEX_ADD_PROXY, OCEX_REGISTER,
    OCEX_REMOVE_PROXY, RUNTIME_SPEC_VERSION, RUNTIME_TRANSACTION_VERSION,
    SUBSRATEE_REGISTRY_MODULE,
};
use core::ops::Deref;
use log::*;
use polkadex_sgx_primitives::types::SignedOrder;
use polkadex_sgx_primitives::{AssetId, PolkadexAccount};
use rpc::author::{hash::TrustedOperationOrHash, Author, AuthorApi};
use rpc::worker_api_direct;
use rpc::{api::SideChainApi, basic_pool::BasicPool};
use sgx_externalities::SgxExternalitiesTypeTrait;
use sgx_types::{sgx_epid_group_id_t, sgx_status_t, sgx_target_info_t, SgxResult};
use sp_core::{blake2_256, crypto::Pair, H256};
use sp_finality_grandpa::VersionedAuthorityList;
use sp_runtime::OpaqueExtrinsic;
use sp_runtime::{generic::SignedBlock, traits::Header as HeaderT};
use std::collections::HashMap;
use std::slice;
use std::sync::Arc;
use std::sync::{SgxMutex, SgxMutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};
use std::untrusted::time::SystemTimeEx;
use std::vec::Vec;
use substrate_api_client::compose_extrinsic_offline;
use substrate_api_client::extrinsic::xt_primitives::UncheckedExtrinsicV4;
use substratee_node_primitives::{
    CallWorkerFn, OCEXAddProxyFn, OCEXDepositFn, OCEXRegisterFn, OCEXRemoveProxyFn, OCEXWithdrawFn,
    ShieldFundsFn,
};
use substratee_stf::sgx::{shards_key_hash, storage_hashes_to_update_per_shard, OpaqueCall};
use substratee_stf::State as StfState;
use substratee_stf::{
    AccountId, Getter, ShardIdentifier, Stf, TrustedCall, TrustedCallSigned, TrustedGetterSigned,
};
use substratee_worker_primitives::block::{
    Block as SidechainBlock, SignedBlock as SignedSidechainBlock, StatePayload,
};
use substratee_worker_primitives::BlockHash;
use utils::write_slice_and_whitespace_pad;

mod aes;
mod attestation;
pub mod cert;
mod constants;
mod ed25519;
mod happy_path;
pub mod hex;
mod io;
mod ipfs;
pub mod polkadex_nonce_storage;
pub mod openfinex;
mod polkadex;
mod polkadex_balance_storage;
pub mod polkadex_cache;
mod polkadex_gateway;
mod polkadex_orderbook_storage;
pub mod rpc;
mod rsa3072;
mod ss58check;
mod state;
mod test_orderbook_storage;
mod test_polkadex_balance_storage;
mod test_polkadex_gateway;
mod test_proxy;
pub mod tests;
pub mod tls_ra;
pub mod top_pool;
mod utils;

pub const CERTEXPIRYDAYS: i64 = 90i64;

#[derive(Debug, Clone, PartialEq)]
pub enum Timeout {
    Call,
    Getter,
}

pub type Hash = sp_core::H256;
type BPool = BasicPool<SideChainApi<Block>, Block>;

#[no_mangle]
pub unsafe extern "C" fn init() -> sgx_status_t {
    // initialize the logging environment in the enclave
    env_logger::init();

    if let Err(status) = ed25519::create_sealed_if_absent() {
        return status;
    }

    let signer = match ed25519::unseal_pair() {
        Ok(pair) => pair,
        Err(status) => return status,
    };
    info!(
        "[Enclave initialized] Ed25519 prim raw : {:?}",
        signer.public().0
    );

    if let Err(status) = rsa3072::create_sealed_if_absent() {
        return status;
    }

    // create the aes key that is used for state encryption such that a key is always present in tests.
    // It will be overwritten anyway if mutual remote attastation is performed with the primary worker
    if let Err(status) = aes::create_sealed_if_absent() {
        return status;
    }

    // for debug purposes, list shards. no problem to panic if fails
    let shards = state::list_shards().unwrap();
    debug!("found the following {} shards on disk:", shards.len());
    for s in shards {
        debug!("{}", s.encode().to_base58())
    }
    //shards.into_iter().map(|s| debug!("{}", s.encode().to_base58()));

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn get_rsa_encryption_pubkey(
    pubkey: *mut u8,
    pubkey_size: u32,
) -> sgx_status_t {
    let rsa_pubkey = match rsa3072::unseal_pubkey() {
        Ok(key) => key,
        Err(status) => return status,
    };

    let rsa_pubkey_json = match serde_json::to_string(&rsa_pubkey) {
        Ok(k) => k,
        Err(x) => {
            println!(
                "[Enclave] can't serialize rsa_pubkey {:?} {}",
                rsa_pubkey, x
            );
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let pubkey_slice = slice::from_raw_parts_mut(pubkey, pubkey_size as usize);
    write_slice_and_whitespace_pad(pubkey_slice, rsa_pubkey_json.as_bytes().to_vec());

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn get_ecc_signing_pubkey(pubkey: *mut u8, pubkey_size: u32) -> sgx_status_t {
    if let Err(status) = ed25519::create_sealed_if_absent() {
        return status;
    }

    let signer = match ed25519::unseal_pair() {
        Ok(pair) => pair,
        Err(status) => return status,
    };
    debug!("Restored ECC pubkey: {:?}", signer.public());

    let pubkey_slice = slice::from_raw_parts_mut(pubkey, pubkey_size as usize);
    pubkey_slice.clone_from_slice(&signer.public());

    sgx_status_t::SGX_SUCCESS
}

fn create_extrinsics(
    validator: LightValidation,
    calls_buffer: Vec<OpaqueCall>,
    mut _nonce: u32,
) -> SgxResult<Vec<Vec<u8>>> {
    // get information for composing the extrinsic
    let signer = ed25519::unseal_pair()?;
    debug!("Restored ECC pubkey: {:?}", signer.public().clone());

    let mut nonce = polkadex_nonce_storage::lock_storage_and_get_nonce(signer.public().clone().into()).unwrap().nonce.unwrap(); //TODO: Error handling

    let extrinsics_buffer: Vec<Vec<u8>> = calls_buffer
        .into_iter()
        .map(|call| {
            let xt = compose_extrinsic_offline!(
                signer.clone(),
                call,
                nonce,
                Era::Immortal,
                validator.genesis_hash(validator.num_relays).unwrap(),
                validator.genesis_hash(validator.num_relays).unwrap(),
                RUNTIME_SPEC_VERSION,
                RUNTIME_TRANSACTION_VERSION
            )
            .encode();
            nonce += 1;
            xt
        })
        .collect();

    // update nonce storage
    polkadex_nonce_storage::lock_and_update_nonce(nonce, signer.public().into()).unwrap(); //TODO: Error handling

    Ok(extrinsics_buffer)
}

#[no_mangle]
pub unsafe extern "C" fn get_state(
    trusted_op: *const u8,
    trusted_op_size: u32,
    shard: *const u8,
    shard_size: u32,
    value: *mut u8,
    value_size: u32,
) -> sgx_status_t {
    let shard = ShardIdentifier::from_slice(slice::from_raw_parts(shard, shard_size as usize));
    let mut trusted_op_slice = slice::from_raw_parts(trusted_op, trusted_op_size as usize);
    let value_slice = slice::from_raw_parts_mut(value, value_size as usize);
    let getter = Getter::decode(&mut trusted_op_slice).unwrap();

    if let Getter::trusted(trusted_getter_signed) = getter.clone() {
        debug!("verifying signature of TrustedGetterSigned");
        if let false = trusted_getter_signed.verify_signature() {
            error!("bad signature");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }

    if !state::exists(&shard) {
        info!("Initialized new shard that was queried chain: {:?}", shard);
        if let Err(e) = state::init_shard(&shard) {
            return e;
        }
    }

    let mut state = match state::load(&shard) {
        Ok(s) => s,
        Err(status) => return status,
    };

    let validator = match io::light_validation::unseal() {
        Ok(val) => val,
        Err(e) => return e,
    };

    let latest_header = validator
        .latest_finalized_header(validator.num_relays)
        .unwrap();

    // FIXME: not sure we will ever need this as we are querying trusted state, not onchain state
    // i.e. demurrage could be correctly applied with this, but the client could do that too.
    debug!("Update STF storage!");
    let requests: Vec<WorkerRequest> = Stf::get_storage_hashes_to_update_for_getter(&getter)
        .into_iter()
        .map(|key| WorkerRequest::ChainStorage(key, Some(latest_header.hash())))
        .collect();

    if !requests.is_empty() {
        let responses: Vec<WorkerResponse<Vec<u8>>> = match worker_request(requests) {
            Ok(resp) => resp,
            Err(e) => return e,
        };

        let update_map = match verify_worker_responses(responses, latest_header) {
            Ok(map) => map,
            Err(e) => return e,
        };

        Stf::update_storage(&mut state, &update_map);
    }

    debug!("calling into STF to get state");
    let value_opt = Stf::get_state(&mut state, getter);

    debug!("returning getter result");
    write_slice_and_whitespace_pad(value_slice, value_opt.encode());

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn init_chain_relay(
    genesis_header: *const u8,
    genesis_header_size: usize,
    authority_list: *const u8,
    authority_list_size: usize,
    authority_proof: *const u8,
    authority_proof_size: usize,
    latest_header: *mut u8,
    latest_header_size: usize,
) -> sgx_status_t {
    info!("Initializing Chain Relay!");

    let mut header = slice::from_raw_parts(genesis_header, genesis_header_size);
    let latest_header_slice = slice::from_raw_parts_mut(latest_header, latest_header_size);
    let mut auth = slice::from_raw_parts(authority_list, authority_list_size);
    let mut proof = slice::from_raw_parts(authority_proof, authority_proof_size);

    let header = match Header::decode(&mut header) {
        Ok(h) => h,
        Err(e) => {
            error!("Decoding Header failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let auth = match VersionedAuthorityList::decode(&mut auth) {
        Ok(a) => a,
        Err(e) => {
            error!("Decoding VersionedAuthorityList failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let proof = match StorageProof::decode(&mut proof) {
        Ok(h) => h,
        Err(e) => {
            error!("Decoding Header failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    match io::light_validation::read_or_init_validator(header, auth, proof) {
        Ok(header) => write_slice_and_whitespace_pad(latest_header_slice, header.encode()),
        Err(e) => return e,
    }

    // Initializes the Order Nonce
    polkadex_gateway::initialize_polkadex_gateway();
    info!(" Polkadex Gateway Nonces and Cache Initialized");

    if let Err(e) = polkadex_nonce_storage::create_in_memory_nonce_storage() {
        error!("Creating in memory nonce storage failed. Error: {:?}", e);
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    if let Err(e) = polkadex_balance_storage::create_in_memory_balance_storage() {
        error!("Creating in memory balance storage failed. Error: {:?}", e);
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn accept_pdex_accounts(
    pdex_accounts: *const u8,
    pdex_accounts_size: usize,
) -> sgx_status_t {
    let mut pdex_accounts_slice = slice::from_raw_parts(pdex_accounts, pdex_accounts_size);

    let polkadex_accounts: Vec<PolkadexAccount> = match Decode::decode(&mut pdex_accounts_slice) {
        Ok(b) => b,
        Err(e) => {
            error!("Decoding signed accounts failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let validator = match io::light_validation::unseal() {
        Ok(v) => v,
        Err(e) => return e,
    };
    let latest_header = validator
        .latest_finalized_header(validator.num_relays)
        .unwrap();

    if let Err(status) =
        polkadex::verify_pdex_account_read_proofs(latest_header, polkadex_accounts.clone())
    {
        return status;
    }

    if let Err(_) = polkadex::create_in_memory_account_storage(polkadex_accounts) {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn load_orders_to_memory(
    orders: *const u8,
    orders_size: usize,
) -> sgx_status_t {
    let mut orders_slice = slice::from_raw_parts(orders, orders_size);

    let signed_orders: Vec<SignedOrder> = match Decode::decode(&mut orders_slice) {
        Ok(b) => b,
        Err(e) => {
            error!("Decoding signed orders failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    if let Err(status) =
        polkadex_orderbook_storage::create_in_memory_orderbook_storage(signed_orders)
    {
        return status;
    };

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn sync_chain(
    blocks_to_sync: *const u8,
    blocks_to_sync_size: usize,
    nonce: *const u32,
) -> sgx_status_t {
    // FIXME: This design needs some more thoughts.
    // Proposal: Lock nonce handler storage while syncing, and give free after syncing?
    // otherwise some extrsincs have high chance of being invalid.. not really good
    // update nonce storage
    //if let Err(e) = polkadex_nonce_storage::lock_and_update_nonce(*nonce) {
    //    error!("Locking and updating nonce failed. Error: {:?}", e);
    //};

    let mut blocks_to_sync_slice = slice::from_raw_parts(blocks_to_sync, blocks_to_sync_size);

    let blocks_to_sync: Vec<SignedBlock<Block>> = match Decode::decode(&mut blocks_to_sync_slice) {
        Ok(b) => b,
        Err(e) => {
            error!("Decoding signed blocks failed. Error: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let mut validator = match io::light_validation::unseal() {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut calls = Vec::<OpaqueCall>::new();

    debug!("Syncing chain relay!");
    if !blocks_to_sync.is_empty() {
        for signed_block in blocks_to_sync.into_iter() {
            validator
                .check_xt_inclusion(validator.num_relays, &signed_block.block)
                .unwrap(); // panic can only happen if relay_id does not exist
            if let Err(e) = validator.submit_simple_header(
                validator.num_relays,
                signed_block.block.header.clone(),
                signed_block.justifications.clone(),
            ) {
                error!("Block verification failed. Error : {:?}", e);
                return sgx_status_t::SGX_ERROR_UNEXPECTED;
            }

            if update_states(signed_block.block.header.clone()).is_err() {
                error!("Error performing state updates upon block import");
                return sgx_status_t::SGX_ERROR_UNEXPECTED;
            }

            // execute indirect calls, incl. shielding and unshielding
            match scan_block_for_relevant_xt(&signed_block.block) {
                // push shield funds to opaque calls
                Ok(c) => calls.extend(c.into_iter()),
                Err(_) => error!("Error executing relevant extrinsics"),
            };
            // compose indirect block confirmation
            let xt_block = [SUBSRATEE_REGISTRY_MODULE, BLOCK_CONFIRMED];
            let genesis_hash = validator.genesis_hash(validator.num_relays).unwrap();
            let block_hash = signed_block.block.header.hash();
            let prev_state_hash = signed_block.block.header.parent_hash();
            calls.push(OpaqueCall(
                (xt_block, genesis_hash, block_hash, prev_state_hash.encode()).encode(),
            ));
        }
    }
    // get header of last block
    let latest_onchain_header: Header = validator
        .latest_finalized_header(validator.num_relays)
        .unwrap();
    // execute pending calls from operation pool and create block
    // (one per shard) as opaque call with block confirmation
    let signed_blocks: Vec<SignedSidechainBlock> =
        match execute_top_pool_calls(latest_onchain_header) {
            Ok((confirm_calls, signed_blocks)) => {
                calls.extend(confirm_calls.into_iter());
                signed_blocks
            }
            Err(_) => return sgx_status_t::SGX_ERROR_UNEXPECTED,
        };

    let extrinsics = match create_extrinsics(validator.clone(), calls, *nonce) {
        Ok(xt) => xt,
        Err(_) => return sgx_status_t::SGX_ERROR_UNEXPECTED,
    };

    // store extrinsics in chain relay for finalization check
    for xt in extrinsics.iter() {
        validator
            .submit_xt_to_be_included(
                validator.num_relays,
                OpaqueExtrinsic::from_bytes(xt.as_slice()).unwrap(),
            )
            .unwrap();
    }

    if io::light_validation::seal(validator).is_err() {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    // ocall to worker to store signed block and send block confirmation
    if let Err(_e) = send_block_and_confirmation(extrinsics, signed_blocks) {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }

    sgx_status_t::SGX_SUCCESS
}

fn send_block_and_confirmation(
    confirmations: Vec<Vec<u8>>,
    signed_blocks: Vec<SignedSidechainBlock>,
) -> SgxResult<()> {
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let res = unsafe {
        ocall_send_block_and_confirmation(
            &mut rt as *mut sgx_status_t,
            confirmations.encode().as_ptr(),
            confirmations.encode().len() as u32,
            signed_blocks.encode().as_ptr(),
            signed_blocks.encode().len() as u32,
        )
    };

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(rt);
    }

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(res);
    }

    Ok(())
}

fn get_stf_state(
    trusted_getter_signed: TrustedGetterSigned,
    shard: ShardIdentifier,
) -> Option<Vec<u8>> {
    debug!("verifying signature of TrustedGetterSigned");
    if let false = trusted_getter_signed.verify_signature() {
        error!("bad signature");
        return None;
    }

    if !state::exists(&shard) {
        info!("Initialized new shard that was queried chain: {:?}", shard);
        if let Err(e) = state::init_shard(&shard) {
            error!("Error initialising shard {:?} state: Error: {:?}", shard, e);
            return None;
        }
    }

    let mut state = match state::load(&shard) {
        Ok(s) => s,
        Err(e) => {
            error!("Error loading shard {:?}: Error: {:?}", shard, e);
            return None;
        }
    };

    debug!("calling into STF to get state");
    Stf::get_state(&mut state, trusted_getter_signed.into())
}

fn execute_top_pool_calls(
    latest_onchain_header: Header,
) -> SgxResult<(Vec<OpaqueCall>, Vec<SignedSidechainBlock>)> {
    debug!("Executing pending pool operations");
    let mut calls = Vec::<OpaqueCall>::new();
    let mut blocks = Vec::<SignedSidechainBlock>::new();
    {
        // load top pool
        let pool_mutex: &SgxMutex<BPool> = match rpc::worker_api_direct::load_top_pool() {
            Some(mutex) => mutex,
            None => {
                error!("Could not get mutex to pool");
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        };
        let pool_guard: SgxMutexGuard<BPool> = pool_mutex.lock().unwrap();
        let pool: Arc<&BPool> = Arc::new(pool_guard.deref());
        let author: Arc<Author<&BPool>> = Arc::new(Author::new(pool.clone()));

        // get all shards
        let shards = state::list_shards()?;

        // Handle trusted getters
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut is_done = false;
        for shard in shards.clone().into_iter() {
            // retrieve trusted operations from pool
            let trusted_getters = match author.get_pending_tops_separated(shard) {
                Ok((_, getters)) => getters,
                Err(_) => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
            };
            for trusted_getter_signed in trusted_getters.into_iter() {
                // get state
                let value_opt = get_stf_state(trusted_getter_signed.clone(), shard);
                // get hash
                let hash_of_getter = author.hash_of(&trusted_getter_signed.into());
                // let client know of current state
                if worker_api_direct::send_state(hash_of_getter, value_opt).is_err() {
                    error!("Could not get state from stf");
                }
                // remove getter from pool
                if let Err(e) = author.remove_top(
                    vec![TrustedOperationOrHash::Hash(hash_of_getter)],
                    shard,
                    false,
                ) {
                    error!(
                        "Error removing trusted operation from top pool: Error: {:?}",
                        e
                    );
                }
                // Check time
                if time_is_overdue(Timeout::Getter, start_time) {
                    is_done = true;
                    break;
                }
            }
            if is_done {
                break;
            }
        }

        // Handle trusted calls
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut is_done = false;
        for shard in shards.into_iter() {
            let mut call_hashes = Vec::<H256>::new();

            // load state before executing any calls
            let mut state = if state::exists(&shard) {
                state::load(&shard)?
            } else {
                state::init_shard(&shard)?;
                Stf::init_state()
            };
            // save the state hash before call executions
            // (needed for block composition)
            let prev_state_hash = state::hash_of(state.state.clone())?;

            // retrieve trusted operations from pool
            let trusted_calls = match author.get_pending_tops_separated(shard) {
                Ok((calls, _)) => calls,
                Err(_) => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
            };
            debug!("Got following trusted calls from pool: {:?}", trusted_calls);
            // call execution
            for trusted_call_signed in trusted_calls.into_iter() {
                match handle_trusted_worker_call(
                    &mut calls,
                    &mut state,
                    trusted_call_signed,
                    latest_onchain_header.clone(),
                    shard,
                    Some(author.clone()),
                ) {
                    Ok(hashes) => {
                        if let Some((_, operation_hash)) = hashes {
                            call_hashes.push(operation_hash)
                        }
                    }
                    Err(e) => error!("Error performing worker call: Error: {:?}", e),
                };
                // Check time
                if time_is_overdue(Timeout::Call, start_time) {
                    is_done = true;
                    break;
                }
            }
            // create new block
            match compose_block_and_confirmation(
                latest_onchain_header.clone(),
                call_hashes,
                shard,
                prev_state_hash,
                &mut state,
            ) {
                Ok((block_confirm, signed_block)) => {
                    calls.push(block_confirm);
                    blocks.push(signed_block.clone());

                    // Notify watching clients of InSidechainBlock
                    let composed_block = signed_block.block();
                    let block_hash: BlockHash = blake2_256(&composed_block.encode()).into();
                    pool.pool()
                        .validated_pool()
                        .on_block_created(composed_block.signed_top_hashes(), block_hash);
                }
                Err(e) => error!("Could not compose block confirmation: {:?}", e),
            }
            // save updated state after call executions
            let _new_state_hash = state::write(state.clone(), &shard)?;

            if is_done {
                break;
            }
        }
    }

    Ok((calls, blocks))
}

/// Checks if the time of call execution or getter is overdue
/// Returns true if specified time is exceeded
pub fn time_is_overdue(timeout: Timeout, start_time: i64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let max_time_ms: i64 = match timeout {
        Timeout::Call => CALLTIMEOUT,
        Timeout::Getter => GETTERTIMEOUT,
    };
    (now - start_time) * 1000 >= max_time_ms
}

/// Composes a sidechain block of a shard
pub fn compose_block_and_confirmation(
    latest_onchain_header: Header,
    top_call_hashes: Vec<H256>,
    shard: ShardIdentifier,
    state_hash_apriori: H256,
    state: &mut StfState,
) -> SgxResult<(OpaqueCall, SignedSidechainBlock)> {
    let signer_pair = ed25519::unseal_pair()?;
    let layer_one_head = latest_onchain_header.hash();

    let block_number = match Stf::get_sidechain_block_number(state) {
        Some(number) => number + 1,
        None => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
    };
    Stf::update_sidechain_block_number(state, block_number);

    let block_number: u64 = block_number; //FIXME! Should be either u64 or u32! Not both..
    let parent_hash = match Stf::get_last_block_hash(state) {
        Some(hash) => hash,
        None => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
    };
    // hash previous of state
    let state_hash_aposteriori = state::hash_of(state.state.clone())?;
    let state_update = state.state_diff.clone().encode();

    // create encrypted payload
    let mut payload: Vec<u8> =
        StatePayload::new(state_hash_apriori, state_hash_aposteriori, state_update).encode();
    aes::de_or_encrypt(&mut payload)?;

    let block = SidechainBlock::construct_block(
        signer_pair.public().into(),
        block_number,
        parent_hash,
        layer_one_head,
        shard,
        top_call_hashes,
        payload,
    );

    let signed_block = block.sign(&signer_pair);

    let block_hash = blake2_256(&block.encode());
    debug!("Block hash 0x{}", hex::encode_hex(&block_hash));
    Stf::update_last_block_hash(state, block_hash.into());

    let xt_block = [SUBSRATEE_REGISTRY_MODULE, BLOCK_CONFIRMED];
    let opaque_call =
        OpaqueCall((xt_block, shard, block_hash, state_hash_aposteriori.encode()).encode());
    Ok((opaque_call, signed_block))
}

pub fn update_states(header: Header) -> SgxResult<()> {
    debug!("Update STF storage upon block import!");
    let requests: Vec<WorkerRequest> = Stf::storage_hashes_to_update_on_block()
        .into_iter()
        .map(|key| WorkerRequest::ChainStorage(key, Some(header.hash())))
        .collect();

    if requests.is_empty() {
        return Ok(());
    }

    // global requests they are the same for every shard
    let responses: Vec<WorkerResponse<Vec<u8>>> = worker_request(requests)?;
    let update_map = verify_worker_responses(responses, header.clone())?;
    // look for new shards an initialize them
    if let Some(maybe_shards) = update_map.get(&shards_key_hash()) {
        match maybe_shards {
            Some(shards) => {
                let shards: Vec<ShardIdentifier> = Decode::decode(&mut shards.as_slice())
                    .sgx_error_with_log("error decoding shards")?;

                for s in shards {
                    if !state::exists(&s) {
                        info!("Initialized new shard that was found on chain: {:?}", s);
                        state::init_shard(&s)?;
                    }
                    // per shard (cid) requests
                    let per_shard_request = storage_hashes_to_update_per_shard(&s)
                        .into_iter()
                        .map(|key| WorkerRequest::ChainStorage(key, Some(header.hash())))
                        .collect();

                    let responses: Vec<WorkerResponse<Vec<u8>>> =
                        worker_request(per_shard_request)?;
                    let per_shard_update_map = verify_worker_responses(responses, header.clone())?;

                    let mut state = state::load(&s)?;
                    Stf::update_storage(&mut state, &per_shard_update_map);
                    Stf::update_storage(&mut state, &update_map);

                    // block number is purged from the substrate state so it can't be read like other storage values
                    Stf::update_layer_one_block_number(&mut state, header.number);

                    state::write(state, &s)?;
                }
            }
            None => debug!("No shards are on the chain yet"),
        };
    };
    Ok(())
}

/// Scans blocks for extrinsics that ask the enclave to execute some actions.
/// Executes indirect invocation calls, aswell as shielding and unshielding calls
/// Returns all unshielding call confirmations as opaque calls
pub fn scan_block_for_relevant_xt(block: &Block) -> SgxResult<Vec<OpaqueCall>> {
    debug!("Scanning block {} for relevant xt", block.header.number());
    let mut opaque_calls = Vec::<OpaqueCall>::new();
    for xt_opaque in block.extrinsics.iter() {
        if let Ok(xt) =
            UncheckedExtrinsicV4::<ShieldFundsFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            // confirm call decodes successfully as well
            if xt.function.0 == [SUBSRATEE_REGISTRY_MODULE, SHIELD_FUNDS] {
                if let Err(e) = handle_shield_funds_xt(&mut opaque_calls, xt) {
                    error!("Error performing shield funds. Error: {:?}", e);
                }
            }
        };

        // Polkadex OCEX Register
        if let Ok(xt) =
            UncheckedExtrinsicV4::<OCEXRegisterFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            // confirm call decodes successfully as well
            if xt.function.0 == [OCEX_MODULE, OCEX_REGISTER] {
                if let Err(e) = handle_ocex_register(&mut opaque_calls, xt) {
                    error!("Error performing ocex register. Error: {:?}", e);
                }
            }
        }
        // Polkadex OCEX Add Proxy
        if let Ok(xt) =
            UncheckedExtrinsicV4::<OCEXAddProxyFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            // confirm call decodes successfully as well
            if xt.function.0 == [OCEX_MODULE, OCEX_ADD_PROXY] {
                if let Err(e) = handle_ocex_add_proxy(&mut opaque_calls, xt) {
                    error!("Error performing ocex add proxy. Error: {:?}", e);
                }
            }
        }
        // Polkadex OCEX Remove Proxy
        if let Ok(xt) =
            UncheckedExtrinsicV4::<OCEXRemoveProxyFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            // confirm call decodes successfully as well
            if xt.function.0 == [OCEX_MODULE, OCEX_REMOVE_PROXY] {
                if let Err(e) = handle_ocex_remove_proxy(&mut opaque_calls, xt) {
                    error!("Error performing ocex remove proxy. Error: {:?}", e);
                }
            }
        }

        // Polkadex OCEX Withdraw
        if let Ok(xt) =
            UncheckedExtrinsicV4::<OCEXWithdrawFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            // confirm call decodes successfully as well
            if xt.function.0 == [OCEX_MODULE, OCEX_WITHDRAW] {
                if let Err(e) = handle_ocex_withdraw(&mut opaque_calls, xt) {
                    error!("Error performing ocex withdraw. Error: {:?}", e);
                }
            }
        }

        // Polkadex OCEX Deposit
        if let Ok(xt) =
            UncheckedExtrinsicV4::<OCEXDepositFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            // confirm call decodes successfully as well
            if xt.function.0 == [OCEX_MODULE, OCEX_DEPOSIT] {
                if let Err(e) = handle_ocex_deposit(&mut opaque_calls, xt) {
                    error!("Error performing ocex deposit. Error: {:?}", e);
                }
            }
        }

        if let Ok(xt) =
            UncheckedExtrinsicV4::<CallWorkerFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            if xt.function.0 == [SUBSRATEE_REGISTRY_MODULE, CALL_WORKER] {
                if let Ok((decrypted_trusted_call, shard)) = decrypt_unchecked_extrinsic(xt) {
                    // load state before executing any calls
                    let mut state = if state::exists(&shard) {
                        state::load(&shard)?
                    } else {
                        state::init_shard(&shard)?;
                        Stf::init_state()
                    };
                    // call execution
                    if let Err(e) = handle_trusted_worker_call(
                        &mut opaque_calls, // necessary for unshielding
                        &mut state,
                        decrypted_trusted_call,
                        block.header.clone(),
                        shard,
                        None,
                    ) {
                        error!("Error performing worker call: Error: {:?}", e);
                    }
                    // save updated state
                    state::write(state, &shard)?;
                }
            }
        }
    }

    Ok(opaque_calls)
}

fn handle_ocex_register(
    _calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXRegisterFn>,
) -> SgxResult<()> {
    let (call, main_acc) = xt.function.clone(); // TODO: what to do in this case
    info!(
        "Found OCEX Register extrinsic in block: \nCall: {:?} \nMain: {:?} ",
        call,
        main_acc.encode().to_base58(),
    );
    polkadex::add_main_account(main_acc.into()).map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)
}

fn handle_ocex_add_proxy(
    _calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXAddProxyFn>,
) -> SgxResult<()> {
    let (call, main_acc, proxy) = xt.function.clone();
    info!(
        "Found OCEX Add Proxy extrinsic in block: \nCall: {:?} \nMain: {:?}  \nProxy Acc: {}",
        call,
        main_acc.encode().to_base58(),
        proxy.encode().to_base58()
    );
    polkadex::add_proxy(main_acc.into(), proxy.into())
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)
}

fn handle_ocex_remove_proxy(
    _calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXRemoveProxyFn>,
) -> SgxResult<()> {
    let (call, main_acc, proxy) = xt.function.clone();
    info!(
        "Found OCEX Remove Proxy extrinsic in block: \nCall: {:?} \nMain: {:?}  \nProxy Acc: {}",
        call,
        main_acc.encode().to_base58(),
        proxy.encode().to_base58()
    );
    polkadex::remove_proxy(main_acc.into(), proxy.into())
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)
}

fn handle_ocex_deposit(
    _calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXDepositFn>,
) -> SgxResult<()> {
    let (call, main_acc, token, amount) = xt.function.clone();
    info!(
        "Found OCEX Deposit extrinsic in block: \nCall: {:?} \nMain: {:?}  \nToken: {:?} \nAmount: {}",
        call,
        main_acc,
        token.encode().to_base58(),
        amount
    );
    if let Err(_) = polkadex_balance_storage::lock_storage_and_deposit(main_acc, token, amount) {
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(())
    }
}

fn handle_ocex_withdraw(
    calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXWithdrawFn>,
) -> SgxResult<()> {
    let (call, main_acc, token, amount) = xt.function.clone();
    info!(
        "Found OCEX Withdraw extrinsic in block: \nCall: {:?} \nMain: {:?}  \nToken: {:?} \nAmount: {}",
        call,
        main_acc.clone().encode().to_base58(), //FIXME @gautham please look into it
        token.encode().to_base58(),
        amount
    );

    match polkadex::check_if_main_account_registered(main_acc.clone().into()) {
        // TODO: Check if proxy is registered since proxy can also invoke a withdrawal
        Ok(exists) => {
            if exists {
                match polkadex_balance_storage::lock_storage_and_withdraw(
                    main_acc.clone(),
                    token.clone(),
                    amount,
                ) {
                    Ok(()) => {
                        // Compose the release extrinsic
                        let xt_block = [OCEX_MODULE, OCEX_RELEASE];
                        calls.push(OpaqueCall((xt_block, token, amount, main_acc).encode()));
                        return Ok(());
                    }
                    Err(_) => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
                }
            } else {
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        }
        Err(_) => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
    }
}

fn execute_ocex_release_extrinsic(acc: AccountId, token: AssetId, amount: u128) -> SgxResult<()> {
    // TODO: compose an ocex::release extrinsic, sign with enclave signing key and send it through ocall
    let validator = match io::light_validation::unseal() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    // Compose the release extrinsic
    let xt_block = [OCEX_MODULE, OCEX_RELEASE];
    let genesis_hash = validator.genesis_hash(validator.num_relays).unwrap();
    let call: OpaqueCall = OpaqueCall((xt_block, token, amount, acc.clone()).encode());

    // Load the enclave's key pair
    let signer = ed25519::unseal_pair()?;
    debug!("Restored ECC pubkey: {:?}", signer.public());

    let nonce = match lock_storage_and_get_nonce(acc.clone()) {
        Ok(nonce) => Ok(nonce),
        Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
    }?;

    let nonce = nonce.nonce.unwrap();
    debug!("using nonce for ocex release: {:?}", nonce);

    let xt: Vec<u8> = compose_extrinsic_offline!(
        signer,
        call,
        nonce,
        Era::Immortal,
        genesis_hash,
        genesis_hash,
        RUNTIME_SPEC_VERSION,
        RUNTIME_TRANSACTION_VERSION
    )
    .encode();

    lock_storage_and_increment_nonce(acc.clone()).unwrap(); //TODO: Error handling
    //nonce_storage.increment();

    send_release_extrinsic(xt)?;
    Ok(())
}

fn handle_shield_funds_xt(
    calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<ShieldFundsFn>,
) -> SgxResult<()> {
    let (call, account_encrypted, amount, shard) = xt.function.clone();
    info!("Found ShieldFunds extrinsic in block: \nCall: {:?} \nAccount Encrypted {:?} \nAmount: {} \nShard: {}",
          call, account_encrypted, amount, shard.encode().to_base58(),
    );

    let mut state = if state::exists(&shard) {
        state::load(&shard)?
    } else {
        state::init_shard(&shard)?;
        Stf::init_state()
    };

    debug!("decrypt the call");
    let rsa_keypair = rsa3072::unseal_pair()?;
    let account_vec = rsa3072::decrypt(&account_encrypted, &rsa_keypair)?;
    let account = AccountId::decode(&mut account_vec.as_slice())
        .sgx_error_with_log("[ShieldFunds] Could not decode account")?;

    if let Err(e) = Stf::execute(
        &mut state,
        TrustedCallSigned::new(
            TrustedCall::balance_shield(account, amount),
            0,                  //nonce
            Default::default(), //don't care about signature here
        ),
        calls,
    ) {
        error!("Error performing Stf::execute. Error: {:?}", e);
        return Ok(());
    }

    let state_hash = state::write(state, &shard)?;

    let xt_call = [SUBSRATEE_REGISTRY_MODULE, CALL_CONFIRMED];
    let call_hash = blake2_256(&xt.encode());
    debug!("Call hash 0x{}", hex::encode_hex(&call_hash));

    calls.push(OpaqueCall(
        (xt_call, shard, call_hash, state_hash.encode()).encode(),
    ));

    Ok(())
}

fn decrypt_unchecked_extrinsic(
    xt: UncheckedExtrinsicV4<CallWorkerFn>,
) -> SgxResult<(TrustedCallSigned, ShardIdentifier)> {
    let (call, request) = xt.function;
    let (shard, cyphertext) = (request.shard, request.cyphertext);
    debug!("Found CallWorker extrinsic in block: \nCall: {:?} \nRequest: \nshard: {}\ncyphertext: {:?}",
           call,
           shard.encode().to_base58(),
           cyphertext
    );

    debug!("decrypt the call");
    let rsa_keypair = rsa3072::unseal_pair()?;
    let request_vec = rsa3072::decrypt(&cyphertext, &rsa_keypair)?;
    match TrustedCallSigned::decode(&mut request_vec.as_slice()) {
        Ok(call) => Ok((call, shard)),
        Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
    }
}

fn handle_trusted_worker_call(
    calls: &mut Vec<OpaqueCall>,
    state: &mut StfState,
    stf_call_signed: TrustedCallSigned,
    header: Header,
    shard: ShardIdentifier,
    author_pointer: Option<Arc<Author<&BPool>>>,
) -> SgxResult<Option<(H256, H256)>> {
    debug!("query mrenclave of self");
    let mrenclave = attestation::get_mrenclave_of_self()?;
    debug!("MRENCLAVE of self is {}", mrenclave.m.to_base58());

    if let false = stf_call_signed.verify_signature(&mrenclave.m, &shard) {
        error!("TrustedCallSigned: bad signature");
        // do not panic here or users will be able to shoot workers dead by supplying a bad signature
        if let Some(author) = author_pointer {
            // remove call as invalid from pool
            let inblock = false;
            author
                .remove_top(
                    vec![TrustedOperationOrHash::Operation(
                        stf_call_signed.into_trusted_operation(true),
                    )],
                    shard,
                    inblock,
                )
                .unwrap();
        }
        return Ok(None);
    }

    // Necessary because chain relay sync may not be up to date
    // see issue #208
    debug!("Update STF storage!");
    let requests = Stf::get_storage_hashes_to_update(&stf_call_signed)
        .into_iter()
        .map(|key| WorkerRequest::ChainStorage(key, Some(header.hash())))
        .collect();

    let responses: Vec<WorkerResponse<Vec<u8>>> = worker_request(requests)?;

    let update_map = verify_worker_responses(responses, header)?;

    Stf::update_storage(state, &update_map);

    debug!("execute STF");
    if let Err(e) = Stf::execute(state, stf_call_signed.clone(), calls) {
        if let Some(author) = author_pointer {
            // remove call as invalid from pool
            let inblock = false;
            author
                .remove_top(
                    vec![TrustedOperationOrHash::Operation(
                        stf_call_signed.into_trusted_operation(true),
                    )],
                    shard,
                    inblock,
                )
                .unwrap();
        }
        error!("Error performing Stf::execute. Error: {:?}", e);

        return Ok(None);
    }

    if let Some(author) = author_pointer {
        // TODO: prune instead of remove_top ? Block needs to be known
        // remove call from pool as valid
        // TODO: move this pruning to after finalization confirmations, not here!
        let inblock = true;
        author
            .remove_top(
                vec![TrustedOperationOrHash::Operation(
                    stf_call_signed.clone().into_trusted_operation(true),
                )],
                shard,
                inblock,
            )
            .unwrap();
    }
    let call_hash = blake2_256(&stf_call_signed.encode());
    let operation = stf_call_signed.into_trusted_operation(true);
    let operation_hash = blake2_256(&operation.encode());
    debug!("Operation hash 0x{}", hex::encode_hex(&operation_hash));
    debug!("Call hash 0x{}", hex::encode_hex(&call_hash));

    Ok(Some((H256::from(call_hash), H256::from(operation_hash))))
}

fn verify_worker_responses(
    responses: Vec<WorkerResponse<Vec<u8>>>,
    header: Header,
) -> SgxResult<HashMap<Vec<u8>, Option<Vec<u8>>>> {
    let mut update_map = HashMap::new();
    for response in responses.iter() {
        match response {
            WorkerResponse::ChainStorage(key, value, proof) => {
                let proof = proof
                    .as_ref()
                    .sgx_error_with_log("No Storage Proof Supplied")?;

                let actual = StorageProofChecker::<<Header as HeaderT>::Hashing>::check_proof(
                    header.state_root,
                    key,
                    proof.to_vec(),
                )
                .sgx_error_with_log("Erroneous StorageProof")?;

                // Todo: Why do they do it like that, we could supply the proof only and get the value from the proof directly??
                if &actual != value {
                    error!("Wrong storage value supplied");
                    return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                }
                update_map.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(update_map)
}

extern "C" {
    pub fn ocall_write_order_to_db(
        ret_val: *mut sgx_status_t,
        order: *const u8,
        order_size: u32,
    ) -> sgx_status_t;

    pub fn ocall_read_ipfs(
        ret_val: *mut sgx_status_t,
        cid: *const u8,
        cid_size: u32,
    ) -> sgx_status_t;

    pub fn ocall_write_ipfs(
        ret_val: *mut sgx_status_t,
        enc_state: *const u8,
        enc_state_size: u32,
        cid: *mut u8,
        cid_size: u32,
    ) -> sgx_status_t;

    pub fn ocall_worker_request(
        ret_val: *mut sgx_status_t,
        request: *const u8,
        req_size: u32,
        response: *mut u8,
        resp_size: u32,
    ) -> sgx_status_t;

    pub fn ocall_sgx_init_quote(
        ret_val: *mut sgx_status_t,
        ret_ti: *mut sgx_target_info_t,
        ret_gid: *mut sgx_epid_group_id_t,
    ) -> sgx_status_t;

    pub fn ocall_send_block_and_confirmation(
        ret_val: *mut sgx_status_t,
        confirmations: *const u8,
        confirmations_size: u32,
        signed_blocks: *const u8,
        signed_blocks_size: u32,
    ) -> sgx_status_t;

    pub fn ocall_send_release_extrinsic(
        ret_val: *mut sgx_status_t,
        xt: *const u8,
        size_xt: u32,
    ) -> sgx_status_t;
}

// TODO: this is redundantly defined in worker/src/main.rs
#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum WorkerRequest {
    ChainStorage(Vec<u8>, Option<Hash>), // (storage_key, at_block)
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum WorkerResponse<V: Encode + Decode> {
    ChainStorage(Vec<u8>, Option<V>, Option<Vec<Vec<u8>>>), // (storage_key, storage_value, storage_proof)
}

fn worker_request<V: Encode + Decode>(
    req: Vec<WorkerRequest>,
) -> SgxResult<Vec<WorkerResponse<V>>> {
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let mut resp: Vec<u8> = vec![0; 4196 * 4];

    let res = unsafe {
        ocall_worker_request(
            &mut rt as *mut sgx_status_t,
            req.encode().as_ptr(),
            req.encode().len() as u32,
            resp.as_mut_ptr(),
            resp.len() as u32,
        )
    };

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(rt);
    }

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(res);
    }
    Ok(Decode::decode(&mut resp.as_slice()).unwrap())
}

fn _write_order_to_disk(order: SignedOrder) -> SgxResult<()> {
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let res = unsafe {
        ocall_write_order_to_db(
            &mut rt as *mut sgx_status_t,
            order.encode().as_ptr(),
            order.encode().len() as u32,
        )
    };

    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(rt);
    }

    if res != sgx_status_t::SGX_SUCCESS {
        return Err(res);
    }
    Ok(())
}

fn send_release_extrinsic(extrinsic: Vec<u8>) -> SgxResult<()> {
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let res = unsafe {
        ocall_send_release_extrinsic(
            &mut rt as *mut sgx_status_t,
            extrinsic.encode().as_ptr(),
            extrinsic.encode().len() as u32,
        )
    };
    if rt != sgx_status_t::SGX_SUCCESS {
        return Err(rt);
    }
    if res != sgx_status_t::SGX_SUCCESS {
        return Err(res);
    }
    Ok(())
}
