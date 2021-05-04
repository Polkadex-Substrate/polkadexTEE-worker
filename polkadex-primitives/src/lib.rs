use codec::{Encode, Decode};
use my_node_runtime::{AccountId,Header};

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct LinkedAccount {
    prev: AccountId,
    pub next: Option<AccountId>,
    pub current: AccountId,
    proxies: vec![]
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct PolkadexAccount {
    pub account: LinkedAccount,
    pub proof: Vec<Vec<u8>>,
}