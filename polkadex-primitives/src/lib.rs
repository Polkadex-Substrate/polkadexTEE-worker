#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(feature = "sgx")]
use sgx_tstd as std;

use std::vec::Vec;

use codec::{Encode, Decode};
use sp_core::crypto::AccountId32;

pub type AccountId = AccountId32;

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct LinkedAccount {
    prev: AccountId,
    pub next: Option<AccountId>,
    pub current: AccountId,
    proxies: Vec<AccountId>
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct PolkadexAccount {
    pub account: LinkedAccount,
    pub proof: Vec<Vec<u8>>,
}