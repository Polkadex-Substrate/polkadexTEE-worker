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

use crate::channel_storage::{create_channel_get_receiver, ChannelType};
use crate::nonce_handler::NonceHandler;
use crate::{
    error::{Error, Result},
    ocall::{
        ocall_component_factory::{OCallComponentFactory, OCallComponentFactoryTrait},
        rpc_ocall::EnclaveRpcOCall,
    },
    utils::{hash_from_slice, UnwrapOrSgxErrorUnexpected},
};
use base58::ToBase58;
use chain_relay::{Block, Header, Validator};
use codec::{alloc::string::String, Decode, Encode};
use core::ops::Deref;
use log::*;
use polkadex_sgx_primitives::{AssetId, PolkadexAccount};
use polkadex_sgx_primitives::{BalancesData, NonceData, OrderbookData};
use rpc::{
    api::SideChainApi,
    author::{hash::TrustedOperationOrHash, Author, AuthorApi},
    basic_pool::BasicPool,
};
use sgx_externalities::SgxExternalitiesTypeTrait;
use sgx_types::{sgx_status_t, SgxResult};
use sp_core::{blake2_256, crypto::Pair, H256};
use sp_finality_grandpa::VersionedAuthorityList;
use sp_runtime::{generic::SignedBlock, traits::Header as HeaderT, OpaqueExtrinsic};
use std::boxed::Box;
use std::{
    borrow::ToOwned,
    collections::HashMap,
    slice,
    sync::{Arc, SgxMutex, SgxMutexGuard},
    time::{SystemTime, UNIX_EPOCH},
    untrusted::time::SystemTimeEx,
    vec::Vec,
};
use substrate_api_client::{
    compose_extrinsic_offline, extrinsic::xt_primitives::UncheckedExtrinsicV4,
};
use substratee_get_storage_verified::GetStorageVerified;
use substratee_node_primitives::{
    CallWorkerFn, OCEXAddProxyFn, OCEXDepositFn, OCEXRegisterFn, OCEXRemoveProxyFn, OCEXWithdrawFn,
    ShieldFundsFn,
};
use substratee_ocall_api::{
    EnclaveAttestationOCallApi, EnclaveOnChainOCallApi, EnclaveRpcOCallApi,
};
use substratee_settings::node::{
    OCEX_ADD_PROXY, OCEX_DEPOSIT, OCEX_MODULE, OCEX_REGISTER, OCEX_RELEASE, OCEX_REMOVE_PROXY,
    OCEX_WITHDRAW,
};
use substratee_settings::{
    enclave::{CALL_TIMEOUT, GETTER_TIMEOUT},
    node::{
        BLOCK_CONFIRMED, CALL_CONFIRMED, CALL_WORKER, REGISTER_ENCLAVE, RUNTIME_SPEC_VERSION,
        RUNTIME_TRANSACTION_VERSION, SHIELD_FUNDS, SUBSTRATEE_REGISTRY_MODULE,
    },
};
use substratee_sgx_crypto::{aes, Aes, StateCrypto};
use substratee_sgx_io::SealedIO;
use substratee_sidechain_primitives::traits::{
    Block as BlockT, SignBlock, SignedBlock as SignedBlockT,
};
use substratee_stf::{
    stf_sgx::OpaqueCall,
    stf_sgx_primitives::{shards_key_hash, storage_hashes_to_update_per_shard},
    AccountId, Getter, ShardIdentifier, State as StfState, State, StatePayload, Stf, TrustedCall,
    TrustedCallSigned, TrustedGetterSigned,
};
use substratee_storage::{StorageEntryVerified, StorageProof};
use substratee_worker_primitives::{
    block::{Block as SidechainBlock, SignedBlock as SignedSidechainBlock},
    BlockHash,
};
use utils::write_slice_and_whitespace_pad;

mod attestation;
pub mod cert;
mod cid;
mod ed25519;
pub mod error;
pub mod hex;
mod io;
mod ipfs;
mod ocall;
pub mod rpc;
mod rsa3072;
mod state;
pub mod tls_ra;
pub mod top_pool;
mod utils;
// added by polkadex
mod accounts_nonce_storage;
pub mod channel_storage;
mod happy_path;
pub mod nonce_handler;
pub mod openfinex;
mod polkadex_balance_storage;
pub mod polkadex_cache;
mod polkadex_gateway;
mod polkadex_orderbook_storage;
pub mod ss58check;
mod test_orderbook_storage;
mod test_polkadex_balance_storage;
mod test_polkadex_gateway;
pub use crate::cid::*;

#[cfg(feature = "test")]
pub mod test;
#[cfg(feature = "test")]
pub mod tests;

use crate::ed25519::Ed25519;
#[cfg(not(feature = "test"))]
use sgx_types::size_t;

// this is a 'dummy' for production mode
#[cfg(not(feature = "test"))]
#[no_mangle]
pub extern "C" fn test_main_entrance() -> size_t {
    unreachable!("Tests are not available when compiled in production mode.")
}

pub const CERTEXPIRYDAYS: i64 = 90i64;

#[derive(Debug, Clone, PartialEq)]
pub enum Timeout {
    Call,
    Getter,
}

pub type Hash = sp_core::H256;
type BPool = BasicPool<SideChainApi<Block>, Block, EnclaveRpcOCall>;

#[no_mangle]
pub unsafe extern "C" fn init() -> sgx_status_t {
    // initialize the logging environment in the enclave
    env_logger::init();

    if let Err(e) = ed25519::create_sealed_if_absent() {
        return e.into();
    }

    let signer = match Ed25519::unseal() {
        Ok(pair) => pair,
        Err(e) => return e.into(),
    };
    info!(
        "[Enclave initialized] Ed25519 prim raw : {:?}",
        signer.public().0
    );

    if let Err(e) = rsa3072::create_sealed_if_absent() {
        return e.into();
    }

    // create the aes key that is used for state encryption such that a key is always present in tests.
    // It will be overwritten anyway if mutual remote attastation is performed with the primary worker
    if let Err(e) = aes::create_sealed_if_absent().map_err(Error::Crypto) {
        return e.into();
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
        Err(e) => return e.into(),
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
    if let Err(e) = ed25519::create_sealed_if_absent() {
        return e.into();
    }

    let signer = match Ed25519::unseal() {
        Ok(pair) => pair,
        Err(e) => return e.into(),
    };
    debug!("Restored ECC pubkey: {:?}", signer.public());

    let pubkey_slice = slice::from_raw_parts_mut(pubkey, pubkey_size as usize);
    pubkey_slice.clone_from_slice(&signer.public());

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn mock_register_enclave_xt(
    genesis_hash: *const u8,
    genesis_hash_size: u32,
    nonce: *const u32,
    w_url: *const u8,
    w_url_size: u32,
    unchecked_extrinsic: *mut u8,
    unchecked_extrinsic_size: u32,
) -> sgx_status_t {
    let genesis_hash_slice = slice::from_raw_parts(genesis_hash, genesis_hash_size as usize);
    let genesis_hash = hash_from_slice(genesis_hash_slice);

    let mut url_slice = slice::from_raw_parts(w_url, w_url_size as usize);
    let url: String = Decode::decode(&mut url_slice).unwrap();
    let extrinsic_slice =
        slice::from_raw_parts_mut(unchecked_extrinsic, unchecked_extrinsic_size as usize);

    let ocall_api = OCallComponentFactory::attestation_api();

    let signer = Ed25519::unseal().unwrap();
    let call = (
        [SUBSTRATEE_REGISTRY_MODULE, REGISTER_ENCLAVE],
        ocall_api
            .get_mrenclave_of_self()
            .map_or_else(|_| Vec::<u8>::new(), |m| m.m.encode()),
        url,
    );

    let xt = compose_extrinsic_offline!(
        signer,
        call,
        *nonce,
        Era::Immortal,
        genesis_hash,
        genesis_hash,
        RUNTIME_SPEC_VERSION,
        RUNTIME_TRANSACTION_VERSION
    )
    .encode();

    write_slice_and_whitespace_pad(extrinsic_slice, xt);
    sgx_status_t::SGX_SUCCESS
}

fn create_extrinsics<V>(
    validator: &V,
    calls_buffer: Vec<OpaqueCall>,
    mut nonce: u32,
) -> Result<Vec<Vec<u8>>>
where
    V: Validator,
{
    // get information for composing the extrinsic
    let signer = Ed25519::unseal()?;
    debug!("Restored ECC pubkey: {:?}", signer.public());

    let mutex = nonce_handler::load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<NonceHandler> = mutex.lock().unwrap();
    let mut nonce = nonce_storage.nonce;

    let extrinsics_buffer: Vec<Vec<u8>> = calls_buffer
        .into_iter()
        .map(|call| {
            let xt = compose_extrinsic_offline!(
                signer.clone(),
                call,
                nonce,
                Era::Immortal,
                validator.genesis_hash(validator.num_relays()).unwrap(),
                validator.genesis_hash(validator.num_relays()).unwrap(),
                RUNTIME_SPEC_VERSION,
                RUNTIME_TRANSACTION_VERSION
            )
            .encode();
            nonce += 1;
            xt
        })
        .collect();

    // update nonce storage
    debug!("Update to new new nonce: {:?}", nonce);
    nonce_storage.update(nonce);

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
            return e.into();
        }
    }

    let mut state = match state::load(&shard) {
        Ok(s) => s,
        Err(e) => return e.into(),
    };

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
        Err(e) => return e.into(),
    }

    // Initializes the Order Nonce
    polkadex_gateway::initialize_polkadex_gateway();
    info!(" Polkadex Gateway Nonces and Cache Initialized");

    if let Err(e) = nonce_handler::create_in_memory_nonce_storage() {
        error!("Creating in memory nonce storage failed. Error: {:?}", e);
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    if let Err(e) = polkadex_balance_storage::create_in_memory_balance_storage() {
        error!("Creating in memory balance storage failed. Error: {:?}", e);
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    if let Err(e) = polkadex_orderbook_storage::create_in_memory_orderbook_storage(vec![]) {
        error!(
            "Creating in memory orderbook storage failed. Error: {:?}",
            e
        );
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }

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
        Err(e) => return e.into(),
    };
    let latest_header = validator
        .latest_finalized_header(validator.num_relays())
        .unwrap();

    if let Err(status) = accounts_nonce_storage::verify_pdex_account_read_proofs(
        latest_header,
        polkadex_accounts.clone(),
    ) {
        return status;
    }

    accounts_nonce_storage::create_in_memory_accounts_and_nonce_storage(polkadex_accounts);

    sgx_status_t::SGX_SUCCESS
}

fn initialize_and_extend_storages(
    balances_data: Vec<BalancesData>,
    nonce_data: Vec<NonceData>,
    orderbook_data: Vec<OrderbookData>,
) -> SgxResult<()> {
    if polkadex_balance_storage::load_balance_storage().is_err() {
        polkadex_balance_storage::create_in_memory_balance_storage()
            .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)?;
    }
    if accounts_nonce_storage::load_registry().is_err() {
        accounts_nonce_storage::create_in_memory_accounts_and_nonce_storage(vec![]);
    }
    error!(">> Orderbook intialized");
    if polkadex_orderbook_storage::load_orderbook().is_err() {
        polkadex_orderbook_storage::create_in_memory_orderbook_storage(vec![])
            .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)?;
    }

    polkadex_balance_storage::lock_storage_extend_from_disk(balances_data)
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)?;
    accounts_nonce_storage::lock_nonce_storage_extend_from_disk(nonce_data)
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)?;
    polkadex_orderbook_storage::lock_storage_extend_from_disk(orderbook_data)
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)?;

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn send_disk_data(encoded_data: *const u8, data_size: usize) -> sgx_status_t {
    let mut data = slice::from_raw_parts(encoded_data, data_size);

    let decoded: polkadex_sgx_primitives::StorageData =
        if let Ok(data) = polkadex_sgx_primitives::StorageData::decode(&mut data) {
            data
        } else {
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        };

    if initialize_and_extend_storages(decoded.balances, decoded.nonce, decoded.orderbook).is_err() {
        sgx_status_t::SGX_ERROR_UNEXPECTED
    } else {
        sgx_status_t::SGX_SUCCESS
    }
}

extern "C" {
    fn ocall_send_nonce(
        ret_val: *mut sgx_status_t,
        account_encoded: *const u8,
        account_size: u32,
        nonce: u32,
    ) -> sgx_status_t;

    pub fn ocall_send_balances(
        ret_val: *mut sgx_status_t,
        account_encoded: *const u8,
        account_size: u32,
        token_encoded: *const u8,
        token_size: u32,
        free: *mut u8,
        reserved: *mut u8,
        balance_size: u32,
    ) -> sgx_status_t;
}

#[no_mangle]
pub unsafe extern "C" fn run_db_thread() -> sgx_status_t {
    let receiver = if let Ok(receiver) = create_channel_get_receiver() {
        receiver
    } else {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    loop {
        match receiver.recv() {
            Ok(ChannelType::Nonce(account, nonce)) => {
                let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
                let slice: &[u8] = account.as_ref();

                ocall_send_nonce(&mut rt as *mut sgx_status_t, slice.as_ptr(), 32, nonce);
            }
            Ok(ChannelType::Balances(account, balances)) => {
                let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
                let account_slice: &[u8] = account.account_id.as_ref();
                let token_slice = account.asset_id.encode();
                let token_slice: &[u8] = token_slice.as_ref();
                let (mut free, mut reserved) = (balances.free.encode(), balances.reserved.encode());

                ocall_send_balances(
                    &mut rt as *mut sgx_status_t,
                    account_slice.as_ptr(),
                    32,
                    token_slice.as_ptr(),
                    token_slice.len() as u32,
                    free.as_mut_ptr(),
                    reserved.as_mut_ptr(),
                    free.len() as u32,
                );
            }
            Ok(ChannelType::Order(order)) => {
                let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

                ocall_write_order_to_db(
                    &mut rt as *mut sgx_status_t,
                    order.encode().as_ptr(),
                    order.encode().len() as u32,
                );
            }
            Err(_) => {
                error!("Failed to receive message from sender");
            }
        }
    }
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
    if let Err(e) = nonce_handler::lock_and_update_nonce(*nonce) {
        error!("Locking and updating nonce failed. Error: {:?}", e);
    };

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
        Err(e) => return e.into(),
    };

    let on_chain_ocall_api = OCallComponentFactory::on_chain_api();

    let mut calls = match sync_blocks_on_chain_relay(
        blocks_to_sync,
        &mut validator,
        on_chain_ocall_api.as_ref(),
    ) {
        Ok(c) => c,
        Err(e) => return e,
    };

    // get header of last block
    let latest_onchain_header: Header = validator
        .latest_finalized_header(validator.num_relays())
        .unwrap();

    // execute pending calls from operation pool and create block
    // (one per shard) as opaque call with block confirmation
    let rpc_ocall_api = OCallComponentFactory::rpc_api();
    let signed_blocks: Vec<SignedSidechainBlock> = match execute_top_pool_calls(
        rpc_ocall_api.as_ref(),
        on_chain_ocall_api.as_ref(),
        latest_onchain_header,
    ) {
        Ok((confirm_calls, signed_blocks)) => {
            calls.extend(confirm_calls.into_iter());
            signed_blocks
        }
        Err(_) => return sgx_status_t::SGX_ERROR_UNEXPECTED,
    };

    let extrinsics = match create_extrinsics(&validator, calls, *nonce) {
        Ok(xt) => xt,
        Err(_) => return sgx_status_t::SGX_ERROR_UNEXPECTED,
    };

    // store extrinsics in chain relay for finalization check
    for xt in extrinsics.iter() {
        validator
            .submit_xt_to_be_included(
                validator.num_relays(),
                OpaqueExtrinsic::from_bytes(xt.as_slice()).unwrap(),
            )
            .unwrap();
    }

    if io::light_validation::seal(validator).is_err() {
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    // ocall to worker to store signed block and send block confirmation
    // send extrinsics to layer 1 block chain, gossip blocks to side-chain
    if let Err(e) = on_chain_ocall_api.send_block_and_confirmation(extrinsics, signed_blocks) {
        error!("Failed to send block and confirmation: {}", e);
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    }

    sgx_status_t::SGX_SUCCESS
}

fn sync_blocks_on_chain_relay<V, O>(
    blocks_to_sync: Vec<SignedBlock<Block>>,
    validator: &mut V,
    on_chain_ocall_api: &O,
) -> SgxResult<Vec<OpaqueCall>>
where
    V: Validator,
    O: EnclaveOnChainOCallApi,
{
    let mut calls = Vec::<OpaqueCall>::new();

    debug!("Syncing chain relay!");
    for signed_block in blocks_to_sync.into_iter() {
        validator
            .check_xt_inclusion(validator.num_relays(), &signed_block.block)
            .unwrap(); // panic can only happen if relay_id does not exist

        if let Err(e) = validator.submit_simple_header(
            validator.num_relays(),
            signed_block.block.header.clone(),
            signed_block.justifications.clone(),
        ) {
            error!("Block verification failed. Error : {:?}", e);
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }

        if update_states(signed_block.block.header.clone(), on_chain_ocall_api).is_err() {
            error!("Error performing state updates upon block import");
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }

        // execute indirect calls, incl. shielding and unshielding
        match scan_block_for_relevant_xt(&signed_block.block, on_chain_ocall_api) {
            // push shield funds to opaque calls
            Ok(c) => calls.extend(c.into_iter()),
            Err(_) => error!("Error executing relevant extrinsics"),
        };

        // compose indirect block confirmation
        let xt_block = [SUBSTRATEE_REGISTRY_MODULE, BLOCK_CONFIRMED];
        let genesis_hash = validator.genesis_hash(validator.num_relays()).unwrap();
        let block_hash = signed_block.block.header.hash();
        let prev_state_hash = signed_block.block.header.parent_hash();
        calls.push(OpaqueCall(
            (xt_block, genesis_hash, block_hash, prev_state_hash.encode()).encode(),
        ));
    }

    Ok(calls)
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

fn execute_top_pool_calls<R, O>(
    rpc_ocall: &R,
    on_chain_ocall: &O,
    latest_onchain_header: Header,
) -> Result<(Vec<OpaqueCall>, Vec<SignedSidechainBlock>)>
where
    R: EnclaveRpcOCallApi,
    O: EnclaveOnChainOCallApi,
{
    debug!("Executing pending pool operations");

    // load top pool
    let pool_mutex: &SgxMutex<BPool> = match rpc::worker_api_direct::load_top_pool() {
        Some(mutex) => mutex,
        None => {
            error!("Could not get mutex to pool");
            return Error::Sgx(sgx_status_t::SGX_ERROR_UNEXPECTED).into();
        }
    };

    let pool_guard: SgxMutexGuard<BPool> = pool_mutex.lock().unwrap();
    let pool: Arc<&BPool> = Arc::new(pool_guard.deref());
    let author: Arc<Author<&BPool>> = Arc::new(Author::new(pool.clone()));

    // get all shards
    let shards = state::list_shards()?;

    // Handle trusted getters
    execute_trusted_getters(rpc_ocall, &author, &shards)?;

    // Handle trusted calls
    let calls_and_blocks =
        execute_trusted_calls(on_chain_ocall, latest_onchain_header, pool, author, shards)?;

    Ok(calls_and_blocks)
}

fn execute_trusted_calls<O>(
    on_chain_ocall: &O,
    latest_onchain_header: Header,
    pool: Arc<&BPool>,
    author: Arc<Author<&BPool>>,
    shards: Vec<H256>,
) -> Result<(Vec<OpaqueCall>, Vec<SignedSidechainBlock>)>
where
    O: EnclaveOnChainOCallApi,
{
    let mut calls = Vec::<OpaqueCall>::new();
    let mut blocks = Vec::<SignedSidechainBlock>::new();
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut is_done = false;
    for shard in shards.into_iter() {
        let mut call_hashes = Vec::<H256>::new();

        // load state before executing any calls
        let mut state = load_initialized_state(&shard)?;
        // save the state hash before call executions
        // (needed for block composition)
        trace!("Getting hash of previous state ..");
        let prev_state_hash = state::hash_of(state.state.clone())?;
        trace!("Loaded hash of previous state: {:?}", prev_state_hash);

        // retrieve trusted operations from pool
        let trusted_calls = author.get_pending_tops_separated(shard)?.0;

        debug!("Got following trusted calls from pool: {:?}", trusted_calls);
        // call execution
        for trusted_call_signed in trusted_calls.into_iter() {
            match handle_trusted_worker_call(
                &mut calls,
                &mut state,
                &trusted_call_signed,
                latest_onchain_header.clone(),
                shard,
                on_chain_ocall,
            ) {
                Ok(hashes) => {
                    let inblock = match hashes {
                        Some((_, operation_hash)) => {
                            call_hashes.push(operation_hash);
                            true
                        }
                        None => {
                            // remove call as invalid from pool
                            false
                        }
                    };

                    // TODO: prune instead of remove_top ? Block needs to be known
                    // TODO: move this pruning to after finalization confirmations, not here!
                    // remove calls from pool (either as valid or invalid)
                    author
                        .remove_top(
                            vec![TrustedOperationOrHash::Operation(Box::new(
                                trusted_call_signed.into_trusted_operation(true),
                            ))],
                            shard,
                            inblock,
                        )
                        .unwrap();
                }
                Err(e) => error!(
                    "Error performing worker call (will not push top hash): Error: {:?}",
                    e
                ),
            };
            // Check time
            if time_is_overdue(Timeout::Call, start_time) {
                is_done = true;
                break;
            }
        }
        // create new block (side-chain)
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

    Ok((calls, blocks))
}

fn load_initialized_state(shard: &H256) -> SgxResult<State> {
    trace!("Loading state from shard {:?}", shard);
    let state = if state::exists(&shard) {
        state::load(&shard)?
    } else {
        state::init_shard(&shard)?;
        Stf::init_state()
    };
    trace!(
        "Sucessfully loaded or initialized state from shard {:?}",
        shard
    );
    Ok(state)
}

fn execute_trusted_getters<R>(
    rpc_ocall: &R,
    author: &Arc<Author<&BPool>>,
    shards: &[H256],
) -> Result<()>
where
    R: EnclaveRpcOCallApi,
{
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut is_done = false;
    for shard in shards.to_owned().into_iter() {
        // retrieve trusted operations from pool
        let trusted_getters = author.get_pending_tops_separated(shard)?.1;
        for trusted_getter_signed in trusted_getters.into_iter() {
            // get state
            let value_opt = get_stf_state(trusted_getter_signed.clone(), shard);
            trace!("Successfully loaded stf state");
            // get hash
            let hash_of_getter = author.hash_of(&trusted_getter_signed.into());
            // let client know of current state
            trace!("Updating client");
            if rpc_ocall.send_state(hash_of_getter, value_opt).is_err() {
                error!("Could not send state to client");
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

    Ok(())
}

/// Checks if the time of call execution or getter is overdue
/// Returns true if specified time is exceeded
pub fn time_is_overdue(timeout: Timeout, start_time: i64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let max_time_ms: i64 = match timeout {
        Timeout::Call => CALL_TIMEOUT,
        Timeout::Getter => GETTER_TIMEOUT,
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
) -> Result<(OpaqueCall, SignedSidechainBlock)> {
    let signer_pair = Ed25519::unseal()?;
    let layer_one_head = latest_onchain_header.hash();

    let block_number = Stf::get_sidechain_block_number(state)
        .map(|n| n + 1)
        .ok_or(Error::Sgx(sgx_status_t::SGX_ERROR_UNEXPECTED))?;

    Stf::update_sidechain_block_number(state, block_number);

    let block_number: u64 = block_number; //FIXME! Should be either u64 or u32! Not both..
    let parent_hash =
        Stf::get_last_block_hash(state).ok_or(Error::Sgx(sgx_status_t::SGX_ERROR_UNEXPECTED))?;

    // hash previous of state
    let state_hash_aposteriori = state::hash_of(state.state.clone())?;
    let state_update = state.state_diff.clone().encode();

    // create encrypted payload
    let mut payload: Vec<u8> =
        StatePayload::new(state_hash_apriori, state_hash_aposteriori, state_update).encode();
    Aes::encrypt(&mut payload)?;

    let block = SidechainBlock::new(
        signer_pair.public().into(),
        block_number,
        parent_hash,
        layer_one_head,
        shard,
        top_call_hashes,
        payload,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    );

    let block_hash = blake2_256(&block.encode());
    let signed_block = block.sign_block(&*signer_pair);

    debug!("Block hash 0x{}", hex::encode_hex(&block_hash));
    Stf::update_last_block_hash(state, block_hash.into());

    let xt_block = [SUBSTRATEE_REGISTRY_MODULE, BLOCK_CONFIRMED];
    let opaque_call =
        OpaqueCall((xt_block, shard, block_hash, state_hash_aposteriori.encode()).encode());
    Ok((opaque_call, signed_block))
}

pub fn update_states<O>(header: Header, on_chain_ocall_api: &O) -> Result<()>
where
    O: EnclaveOnChainOCallApi,
{
    debug!("Update STF storage upon block import!");
    let storage_hashes = Stf::storage_hashes_to_update_on_block();

    if storage_hashes.is_empty() {
        return Ok(());
    }

    // global requests they are the same for every shard
    let update_map = on_chain_ocall_api
        .get_multiple_storages_verified(storage_hashes, &header)
        .map(into_map)?;

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
                    let per_shard_hashes = storage_hashes_to_update_per_shard(&s);
                    let per_shard_update_map = on_chain_ocall_api
                        .get_multiple_storages_verified(per_shard_hashes, &header)
                        .map(into_map)?;

                    let mut state = state::load(&s)?;
                    trace!("Sucessfully loaded state, updating states ...");
                    Stf::update_storage(&mut state, &per_shard_update_map);
                    Stf::update_storage(&mut state, &update_map);

                    // block number is purged from the substrate state so it can't be read like other storage values
                    Stf::update_layer_one_block_number(&mut state, header.number);

                    state::write(state, &s)?;
                }
            }
            None => info!("No shards are on the chain yet"),
        };
    };
    Ok(())
}

/// Scans blocks for extrinsics that ask the enclave to execute some actions.
/// Executes indirect invocation calls, as well as shielding and unshielding calls
/// Returns all unshielding call confirmations as opaque calls
pub fn scan_block_for_relevant_xt<O>(block: &Block, on_chain_ocall: &O) -> Result<Vec<OpaqueCall>>
where
    O: EnclaveOnChainOCallApi,
{
    debug!("Scanning block {} for relevant xt", block.header.number());
    let mut opaque_calls = Vec::<OpaqueCall>::new();
    for xt_opaque in block.extrinsics.iter() {
        // shield funds XT
        if let Ok(xt) =
            UncheckedExtrinsicV4::<ShieldFundsFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            // confirm call decodes successfully as well
            if xt.function.0 == [SUBSTRATEE_REGISTRY_MODULE, SHIELD_FUNDS] {
                if let Err(e) = handle_shield_funds_xt(&mut opaque_calls, xt) {
                    error!("Error performing shieldfunds. Error: {:?}", e);
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
        // call worker XT
        if let Ok(xt) =
            UncheckedExtrinsicV4::<CallWorkerFn>::decode(&mut xt_opaque.encode().as_slice())
        {
            if xt.function.0 == [SUBSTRATEE_REGISTRY_MODULE, CALL_WORKER] {
                if let Ok((decrypted_trusted_call, shard)) = decrypt_unchecked_extrinsic(xt) {
                    // load state before executing any calls
                    let mut state = load_initialized_state(&shard)?;
                    // call execution
                    trace!("Handling trusted worker call of state: {:?}", state);
                    if let Err(e) = handle_trusted_worker_call(
                        &mut opaque_calls, // necessary for unshielding
                        &mut state,
                        &decrypted_trusted_call,
                        block.header.clone(),
                        shard,
                        on_chain_ocall,
                    ) {
                        error!("Error performing worker call: Error: {:?}", e);
                    }
                    // save updated state
                    trace!("Updating state of shard {:?}", shard);
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
    let (call, main_acc) = xt.function; // TODO: what to do in this case
    info!(
        "Found OCEX Register extrinsic in block: \nCall: {:?} \nMain: {:?} ",
        call,
        main_acc.encode().to_base58(),
    );
    accounts_nonce_storage::add_main_account(main_acc)
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)
}

fn handle_ocex_add_proxy(
    _calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXAddProxyFn>,
) -> SgxResult<()> {
    let (call, main_acc, proxy) = xt.function;
    info!(
        "Found OCEX Add Proxy extrinsic in block: \nCall: {:?} \nMain: {:?}  \nProxy Acc: {}",
        call,
        main_acc.encode().to_base58(),
        proxy.encode().to_base58()
    );
    accounts_nonce_storage::add_proxy(main_acc, proxy)
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)
}

fn handle_ocex_remove_proxy(
    _calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXRemoveProxyFn>,
) -> SgxResult<()> {
    let (call, main_acc, proxy) = xt.function;
    info!(
        "Found OCEX Remove Proxy extrinsic in block: \nCall: {:?} \nMain: {:?}  \nProxy Acc: {}",
        call,
        main_acc.encode().to_base58(),
        proxy.encode().to_base58()
    );
    accounts_nonce_storage::remove_proxy(main_acc, proxy)
        .map_err(|_| sgx_status_t::SGX_ERROR_UNEXPECTED)
}

fn handle_ocex_deposit(
    _calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXDepositFn>,
) -> SgxResult<()> {
    let (call, main_acc, token, amount) = xt.function;
    info!(
        "Found OCEX Deposit extrinsic in block: \nCall: {:?} \nMain: {:?}  \nToken: {:?} \nAmount: {}",
        call,
        main_acc,
        token.encode().to_base58(),
        amount
    );
    if polkadex_balance_storage::lock_storage_and_deposit(main_acc, token, amount).is_err() {
        Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
    } else {
        Ok(())
    }
}

fn handle_ocex_withdraw(
    calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<OCEXWithdrawFn>,
) -> SgxResult<()> {
    let (call, main_acc, token, amount) = xt.function;
    info!(
        "Found OCEX Withdraw extrinsic in block: \nCall: {:?} \nMain: {:?}  \nToken: {:?} \nAmount: {}",
        call,
        main_acc.encode().to_base58(), //FIXME @gautham please look into it
        token.encode().to_base58(),
        amount
    );

    match accounts_nonce_storage::check_if_main_account_registered(main_acc.clone()) {
        // TODO: Check if proxy is registered since proxy can also invoke a withdrawal
        Ok(exists) => {
            if exists {
                match polkadex_balance_storage::lock_storage_and_withdraw(
                    main_acc.clone(),
                    token,
                    amount,
                ) {
                    Ok(()) => {
                        // Compose the release extrinsic
                        let xt_block = [OCEX_MODULE, OCEX_RELEASE];
                        calls.push(OpaqueCall((xt_block, token, amount, main_acc).encode()));
                        Ok(())
                    }
                    Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
                }
            } else {
                Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
            }
        }
        Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
    }
}

/// compose an ocex::release extrinsic, sign with enclave signing key and send it through ocall
fn execute_ocex_release_extrinsic(acc: AccountId, token: AssetId, amount: u128) -> SgxResult<()> {
    let validator = match io::light_validation::unseal() {
        Ok(v) => v,
        Err(e) => return Err(e.into()),
    };
    // Compose the release extrinsic
    let xt_block = [OCEX_MODULE, OCEX_RELEASE];
    let genesis_hash = validator.genesis_hash(validator.num_relays()).unwrap();
    let call: OpaqueCall = OpaqueCall((xt_block, token, amount, acc).encode());

    // Load the enclave's key pair
    let signer = Ed25519::unseal()?;
    debug!("Restored ECC pubkey: {:?}", signer.public());

    let mutex = nonce_handler::load_nonce_storage()?;
    let mut nonce_storage: SgxMutexGuard<NonceHandler> = mutex.lock().unwrap();
    let nonce = nonce_storage.nonce;
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
    nonce_storage.increment();

    send_release_extrinsic(xt)?;
    Ok(())
}

fn handle_shield_funds_xt(
    calls: &mut Vec<OpaqueCall>,
    xt: UncheckedExtrinsicV4<ShieldFundsFn>,
) -> Result<()> {
    let (call, account_encrypted, amount, shard) = xt.function.clone();
    info!("Found ShieldFunds extrinsic in block: \nCall: {:?} \nAccount Encrypted {:?} \nAmount: {} \nShard: {}",
        call, account_encrypted, amount, shard.encode().to_base58(),
    );

    let mut state = load_initialized_state(&shard)?;

    debug!("decrypt the call");
    let rsa_keypair = rsa3072::unseal_pair()?;
    let account_vec = rsa3072::decrypt(&account_encrypted, &rsa_keypair)?;
    let account = AccountId::decode(&mut account_vec.as_slice())
        .sgx_error_with_log("[ShieldFunds] Could not decode account")?;
    let root = Stf::get_root(&mut state);
    let nonce = Stf::account_nonce(&mut state, &root);

    if let Err(e) = Stf::execute(
        &mut state,
        TrustedCallSigned::new(
            TrustedCall::balance_shield(root, account, amount),
            nonce,
            Default::default(), //don't care about signature here
        ),
        calls,
    ) {
        error!("Error performing Stf::execute. Error: {:?}", e);
        return Ok(());
    }

    let state_hash = state::write(state, &shard)?;

    let xt_call = [SUBSTRATEE_REGISTRY_MODULE, CALL_CONFIRMED];
    let call_hash = blake2_256(&xt.encode());
    debug!("Call hash 0x{}", hex::encode_hex(&call_hash));

    calls.push(OpaqueCall(
        (xt_call, shard, call_hash, state_hash.encode()).encode(),
    ));

    Ok(())
}

fn decrypt_unchecked_extrinsic(
    xt: UncheckedExtrinsicV4<CallWorkerFn>,
) -> Result<(TrustedCallSigned, ShardIdentifier)> {
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

    Ok(TrustedCallSigned::decode(&mut request_vec.as_slice()).map(|call| (call, shard))?)
}

fn handle_trusted_worker_call<O>(
    calls: &mut Vec<OpaqueCall>,
    state: &mut StfState,
    stf_call_signed: &TrustedCallSigned,
    header: Header,
    shard: ShardIdentifier,
    on_chain_ocall_api: &O,
) -> Result<Option<(H256, H256)>>
where
    O: EnclaveOnChainOCallApi,
{
    debug!("query mrenclave of self");
    let ocall_api = OCallComponentFactory::attestation_api();
    let mrenclave = ocall_api.get_mrenclave_of_self()?;
    debug!("MRENCLAVE of self is {}", mrenclave.m.to_base58());

    if let false = stf_call_signed.verify_signature(&mrenclave.m, &shard) {
        error!("TrustedCallSigned: bad signature");
        // do not panic here or users will be able to shoot workers dead by supplying a bad signature
        return Ok(None);
    }

    // Necessary because chain relay sync may not be up to date
    // see issue #208
    debug!("Update STF storage!");
    let storage_hashes = Stf::get_storage_hashes_to_update(&stf_call_signed);
    let update_map = on_chain_ocall_api
        .get_multiple_storages_verified(storage_hashes, &header)
        .map(into_map)?;
    Stf::update_storage(state, &update_map);

    debug!("execute STF");
    if let Err(e) = Stf::execute(state, stf_call_signed.clone(), calls) {
        error!("Error performing Stf::execute. Error: {:?}", e);
        return Ok(None);
    }

    let call_hash = blake2_256(&stf_call_signed.encode());
    let operation = stf_call_signed.clone().into_trusted_operation(true);
    let operation_hash = blake2_256(&operation.encode());
    debug!("Operation hash 0x{}", hex::encode_hex(&operation_hash));
    debug!("Call hash 0x{}", hex::encode_hex(&call_hash));

    Ok(Some((H256::from(call_hash), H256::from(operation_hash))))
}

// FIXME: these ocalls should probably be moved to the ocall folder
extern "C" {
    pub fn ocall_write_order_to_db(
        ret_val: *mut sgx_status_t,
        order: *const u8,
        order_size: u32,
    ) -> sgx_status_t;

    pub fn ocall_send_release_extrinsic(
        ret_val: *mut sgx_status_t,
        xt: *const u8,
        size_xt: u32,
    ) -> sgx_status_t;

}

pub fn into_map(
    storage_entries: Vec<StorageEntryVerified<Vec<u8>>>,
) -> HashMap<Vec<u8>, Option<Vec<u8>>> {
    storage_entries
        .into_iter()
        .map(|e| e.into_tuple())
        .collect()
}

// fn _write_order_to_disk(order: SignedOrder) -> SgxResult<()> {
//     let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
//
//     let res = unsafe {
//         ocall_write_order_to_db(
//             &mut rt as *mut sgx_status_t,
//             order.encode().as_ptr(),
//             order.encode().len() as u32,
//         )
//     };
//
//     if rt != sgx_status_t::SGX_SUCCESS {
//         return Err(rt);
//     }
//
//     if res != sgx_status_t::SGX_SUCCESS {
//         return Err(res);
//     }
//     Ok(())
// }

/// sends an release extrsinic per ocall to the node
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
