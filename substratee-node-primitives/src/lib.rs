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

#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

use codec::{Decode, Encode};
use polkadex_sgx_primitives::{AccountId, AssetId};
#[cfg(feature = "sgx")]
use sgx_tstd as std;
use sp_core::H256;
use std::vec::Vec;
pub type ShardIdentifier = H256;
pub type BlockNumber = u32;

// Note in the substratee-pallet-registry this is a struct. But for the coded this does not matter.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Request {
    pub shard: ShardIdentifier,
    pub cyphertext: Vec<u8>,
}

pub type SubstrateeConfirmCallFn = ([u8; 2], ShardIdentifier, H256, Vec<u8>);
pub type ShieldFundsFn = ([u8; 2], Vec<u8>, u128, ShardIdentifier);
pub type CallWorkerFn = ([u8; 2], Request);

// Polkadex Types
pub type OCEXRegisterFn = ([u8; 2], AccountId);
pub type OCEXAddProxyFn = ([u8; 2], AccountId, AccountId);
pub type OCEXRemoveProxyFn = ([u8; 2], AccountId, AccountId);
pub type OCEXDepositFn = ([u8; 2], AccountId, AssetId, u128);
pub type OCEXWithdrawFn = ([u8; 2], AccountId, AssetId, u128);

#[cfg(feature = "std")]
pub mod calls {
    pub use my_node_runtime::{
        pallet_substratee_registry::{Enclave, ShardIdentifier},
        AccountId,
    };
    use sp_core::crypto::Pair;
    use sp_runtime::MultiSignature;

    pub fn get_worker_info<P: Pair>(
        api: &substrate_api_client::Api<P>,
        index: u64,
    ) -> Option<Enclave<AccountId, Vec<u8>>>
    where
        MultiSignature: From<P::Signature>,
    {
        api.get_storage_map("SubstrateeRegistry", "EnclaveRegistry", index, None)
            .unwrap()
    }

    pub fn get_worker_for_shard<P: Pair>(
        api: &substrate_api_client::Api<P>,
        shard: &ShardIdentifier,
    ) -> Option<Enclave<AccountId, Vec<u8>>>
    where
        MultiSignature: From<P::Signature>,
    {
        api.get_storage_map("SubstrateeRegistry", "WorkerForShard", shard, None)
            .unwrap()
            .and_then(|w| get_worker_info(&api, w))
    }

    pub fn get_worker_amount<P: Pair>(api: &substrate_api_client::Api<P>) -> Option<u64>
    where
        MultiSignature: From<P::Signature>,
    {
        api.get_storage_value("SubstrateeRegistry", "EnclaveCount", None)
            .unwrap()
    }

    pub fn get_first_worker_that_is_not_equal_to_self<P: Pair>(
        api: &substrate_api_client::Api<P>,
        self_account: &AccountId,
    ) -> Option<Enclave<AccountId, Vec<u8>>>
    where
        MultiSignature: From<P::Signature>,
    {
        // the registry starts indexing its map at one
        for n in 1..=api
            .get_storage_value("SubstrateeRegistry", "EnclaveCount", None)
            .ok()?
            .unwrap()
        {
            let worker = get_worker_info(api, n).unwrap();
            if &worker.pubkey != self_account {
                return Some(worker);
            }
        }
        None
    }

    pub fn get_latest_state<P: Pair>(
        api: &substrate_api_client::Api<P>,
        shard: &ShardIdentifier,
    ) -> Option<[u8; 46]>
    where
        MultiSignature: From<P::Signature>,
    {
        api.get_storage_map("SubstrateeRegistry", "LatestIPFSHash", shard, None)
            .unwrap()
    }
}
