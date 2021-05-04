use chain_relay::Header;
use polkadex_primitives::PolkadexAccounts;

pub fn verify_pdex_account_read_proofs(
    header: Header,
    accounts: Vec<PolkadexAccounts>) -> Result<(),sgx_status_t> {

}