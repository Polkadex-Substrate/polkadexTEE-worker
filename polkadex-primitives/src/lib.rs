use codec::{Encode, Decode};

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct LinkedAccount {
    prev: Vec<u8>,
    next: Option<Vec<u8>>,
    proxies: Vec<Vec<u8>>,
    proof: Option<Vec<Vec<u8>>>
}