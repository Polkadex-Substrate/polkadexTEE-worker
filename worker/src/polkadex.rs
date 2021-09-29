// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use frame_support::PalletId;
// TODO: Fix this import
use my_node_runtime::Header;
use polkadex_sgx_primitives::{AccountId, LinkedAccount, PolkadexAccount};
use sp_core::sr25519;
use sp_runtime::traits::AccountIdConversion;
use substrate_api_client::{rpc::WsRpcClient, Api};

pub fn get_main_accounts(
    header: Header,
    api: &Api<sr25519::Pair, WsRpcClient>,
) -> Vec<PolkadexAccount> {
    // Read the genesis account
    let genesis_account_id: AccountId = PalletId(*b"polka/ga").into_account();

    // Recursively get all the LinkedAccounts and Proofs ( i.e next == None)
    let mut accounts: Vec<PolkadexAccount> = vec![];
    let mut last_account = get_storage_and_proof(genesis_account_id, &header, api);
    accounts.push(last_account.clone());

    while last_account.account.next != None {
        last_account =
            get_storage_and_proof(last_account.account.next.clone().unwrap(), &header, api);
        accounts.push(last_account.clone());
    }
    accounts
}

pub fn get_storage_and_proof(
    acc: AccountId,
    header: &Header,
    api: &Api<sr25519::Pair, WsRpcClient>,
) -> PolkadexAccount {
    let last_acc: LinkedAccount = api
        .get_storage_map(
            "PolkadexOcex",
            "MainAccounts",
            acc.clone(),
            Some(header.hash()),
        )
        .unwrap()
        .unwrap();

    let last_acc_proof: Vec<Vec<u8>> = api
        .get_storage_map_proof::<AccountId, LinkedAccount>(
            "PolkadexOcex",
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
