use codec::{Decode, Encode};
use frame_support::PalletId;
use my_node_runtime::{AccountId, Header};
use sp_core::sr25519;
use sp_core::storage::StorageKey;
use sp_runtime::MultiSignature;
use sp_runtime::traits::{AccountIdConversion, IdentifyAccount, Verify};
use substrate_api_client::Api;

use polkadex_primitives::LinkedAccount;

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct PolkadexAccount {
    pub account: LinkedAccount,
    pub proof: Vec<Vec<u8>>,
}

pub fn get_main_accounts(header: Header, api: &Api<sr25519::Pair>) {
    // Read the genesis account
    let genesis_account_id: AccountID = PalletId(*b"polka/ga").into_account();

    // Recursively get all the LinkedAccounts and Proofs ( i.e next == None)
    let mut accounts: Vec<PolkadexAccount> = vec![];

    let last_acc: LinkedAccount = api.get_storage_map(
        "OCEX",
        "MainAccounts",
        genesis_account_id,
        Some(header.hash()))
        .unwrap()
        .map(|account: LinkedAccount| account.into())
        .unwrap();

    let last_acc_proof: Vec<Vec<u8>> = api.get_storage_map_proof(
        "OCEX",
        "MainAccounts",
        genesis_account_id,
        Some(header.hash()))
        .unwrap()
        .map(|read_proof| read_proof.proof.into_iter().map(|bytes| bytes.0).collect())
        .unwrap();
    accounts.push(PolkadexAccount {
        account: last_acc.clone(),
        proof: last_acc_proof,
    });

    while last_acc.next != None {
        let last_acc: LinkedAccount = api.get_storage_map(
            "OCEX",
            "MainAccounts",
            last_acc.next,
            Some(header.hash()))
            .unwrap()
            .map(|account: LinkedAccount| account.into())
            .unwrap();

        let last_acc_proof: Vec<Vec<u8>> = api.get_storage_map_proof(
            "OCEX",
            "MainAccounts",
            last_acc.next,
            Some(header.hash()))
            .unwrap()
            .map(|read_proof| read_proof.proof.into_iter().map(|bytes| bytes.0).collect())
            .unwrap();
        accounts.push(PolkadexAccount {
            account: last_acc.clone(),
            proof: last_acc_proof,
        });
    }

    // TODO: Encode
}