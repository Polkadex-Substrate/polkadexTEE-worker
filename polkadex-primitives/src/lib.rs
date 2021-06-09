#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

pub mod accounts;
pub mod types;

use codec::{Decode, Encode};
pub use polkadex_primitives::common_types::{AccountId, Balance, Signature};
use sp_core::{H160, H256};
use sp_std::vec::Vec;
pub use polkadex_primitives::assets::AssetId;

pub type ShardIdentifier = H256;
pub type BlockNumber = u32;

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct LinkedAccount {
    pub prev: AccountId,
    pub current: AccountId,
    pub next: Option<AccountId>,
    pub proxies: Vec<AccountId>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct PolkadexAccount {
    pub account: LinkedAccount,
    pub proof: Vec<Vec<u8>>,
}