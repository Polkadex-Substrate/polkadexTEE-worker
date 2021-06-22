use codec::{Decode, Encode};
use polkadex_sgx_primitives::{AccountId, AssetId};


#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PolkadexBalanceKey {
    pub asset_id: AssetId,
    pub account_id: AccountId,
}

impl PolkadexBalanceKey {
    pub fn from(asset_id: AssetId, account_id: AccountId) -> Self {
        Self {
            asset_id,
            account_id,
        }
    }
}