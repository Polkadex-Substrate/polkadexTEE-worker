use codec::{Decode, Encode};
use frame_support::PalletId;
// TODO: Fix this import
use my_node_runtime::{AccountId, Header};
use sp_core::sr25519;
use sp_runtime::traits::{AccountIdConversion, IdentifyAccount, Verify};
use sp_runtime::MultiSignature;
use substrate_api_client::Api;

use polkadex_primitives::{LinkedAccount, PolkadexAccount};

pub fn get_main_accounts(header: Header, api: &Api<sr25519::Pair>) -> Vec<PolkadexAccount> {
    // Read the genesis account
    let genesis_account_id: AccountId = PalletId(*b"polka/ga").into_account();

    // Recursively get all the LinkedAccounts and Proofs ( i.e next == None)
    let mut accounts: Vec<PolkadexAccount> = vec![];
    let mut last_account = get_storage_and_proof(genesis_account_id, &header, api);
    accounts.push(last_account.clone());

    while last_account.account.next != None {
        last_account = get_storage_and_proof(
            last_account.account.next.clone().unwrap().into(),
            &header,
            api,
        );
        accounts.push(last_account.clone());
    }
    accounts
}

pub fn get_storage_and_proof(
    acc: AccountId,
    header: &Header,
    api: &Api<sr25519::Pair>,
) -> PolkadexAccount {
    let last_acc: LinkedAccount = api
        .get_storage_map("OCEX", "MainAccounts", acc.clone(), Some(header.hash()))
        .unwrap()
        .map(|account: LinkedAccount| account)
        .unwrap();

    let last_acc_proof: Vec<Vec<u8>> = api
        .get_storage_map_proof::<AccountId, LinkedAccount>(
            "OCEX",
            "MainAccounts",
            acc,
            Some(header.hash()),
        )
        .unwrap()
        .map(|read_proof| read_proof.proof.into_iter().map(|bytes| bytes.0).collect())
        .unwrap();

    PolkadexAccount {
        account: last_acc,
        proof: last_acc_proof,
    }
}
