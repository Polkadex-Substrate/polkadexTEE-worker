use polkadex_sgx_primitives::{AccountId, AssetId};
use sgx_tstd::vec::Vec;

pub struct ExtrinsicContent {
    account_id: AccountId,
    token: AssetId,
    nonce: u32,
    amount: u128,
}
// TODO KSR We can also use map om place of Vec as Nonce -> ExtrinsicContent
pub struct DepositExtrinsicHelper {
    active_set: Vec<ExtrinsicContent>,
    pending_set: Vec<ExtrinsicContent>,
    approved_nonce: u32,
    unapproved_nonce: u32,
}

impl DepositExtrinsicHelper {
    fn add_active_set_element(&mut self, element: ExtrinsicContent) {
        self.active_set.push(element);
    }

    fn get_unapproved_nonce(&self) -> u32 {
        self.unapproved_nonce
    }

    fn increase_unapproved_nonce(&mut self) {
        self.unapproved_nonce += 1;
    }
}
