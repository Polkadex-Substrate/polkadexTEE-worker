#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

pub mod accounts;
pub mod types;
//use alloc::vec::Vec;
use codec::{Decode, Encode};
pub use polkadex_primitives::common_types::{AccountId, Signature, Balance};
use sp_core::{H160, H256};
use sp_std::vec::Vec;

pub type ShardIdentifier = H256;
pub type BlockNumber = u32;

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct LinkedAccount {
    pub prev: AccountId,
    pub next: Option<AccountId>,
    pub current: AccountId,
    pub proxies: Vec<AccountId>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct PolkadexAccount {
    pub account: LinkedAccount,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Eq, Clone, Encode, Decode, Debug)]
pub enum AssetId {
    POLKADEX,
    DOT, // TODO: Enabled in Parachain upgrade
    CHAINSAFE(H160),
    TOKEN(H160),
    // PARACHAIN(para_id, network, palletInstance, assetID),
}

impl PartialEq for AssetId {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}
