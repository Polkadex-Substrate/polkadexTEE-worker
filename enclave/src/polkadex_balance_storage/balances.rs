use codec::{Decode, Encode};
use polkadex_sgx_primitives::Balance;


#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Balances {
    pub free: Balance,
    pub reserved: Balance,
}

impl Balances {
    pub fn from(free: Balance, reserved: Balance) -> Self {
        Self { free, reserved }
    }
}