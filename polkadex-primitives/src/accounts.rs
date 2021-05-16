use polkadex_primitives::common_types::{AccountId, Signature};

#[cfg(feature = "sgx")]
use sgx_tstd as std;

use std::vec::Vec;

use codec::Encode;
use sp_core::blake2_256;

pub fn get_account(seed: &str) -> AccountId {
    AccountId::new(Vec::from(seed).using_encoded(blake2_256))
}
