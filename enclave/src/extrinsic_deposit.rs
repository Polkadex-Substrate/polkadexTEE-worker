use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_tstd::collections::hash_map::HashMap;
use sgx_tstd::collections::vec_deque::VecDeque;
use sgx_tstd::vec::Vec;

// pub struct ExtrinsicContent {
//     account_id: AccountId,
//     token: AssetId,
//     nonce: u32,
//     amount: u128,
// }
// TODO KSR We can also use map om place of Vec as Nonce -> ExtrinsicContent
pub struct PendingExtrinsicHelper {
    pub active_set: VecDeque<(u32, Vec<u8>)>,
    pub finalized_nonce: u32,
    pub unfinalized_nonce: u32,
}

impl PendingExtrinsicHelper {
    pub fn create() -> Self {
        Self {
            active_set: VecDeque::new(),
            finalized_nonce: 0u32,
            unfinalized_nonce: 0u32,
        }
    }

    pub fn add_active_set_element(&mut self, nonce: u32, element: Vec<u8>) {
        self.active_set.push_front((nonce, element));
    }

    pub fn get_unfinalized_nonce(&self) -> u32 {
        self.unapproved_nonce
    }

    pub fn increase_unfinalized_nonce(&mut self) {
        self.unapproved_nonce += 1;
    }

    pub fn update_finalzied_nonce(&mut self, new_nonce: u32) {
        self.finalized_nonce = new_nonce;
    }

    pub fn update_unfinalzied_nonce(&mut self, new_nonce: u32) {
        self.unfinalized_nonce = new_nonce;
    }
}
