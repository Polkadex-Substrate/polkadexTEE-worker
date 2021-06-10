use polkadex_primitives::common_types::{AccountId};


pub extern crate alloc;
use alloc::vec::Vec;

use codec::Encode;
use sp_core::blake2_256;

pub fn get_account(seed: &str) -> AccountId {
    AccountId::new(Vec::from(seed).using_encoded(blake2_256))
}
