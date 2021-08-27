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

use super::error::Error;
use crate::{accounts_nonce_storage, accounts_nonce_storage::AccountsNonceStorage};
use codec::Encode;
use polkadex_sgx_primitives::AccountId;
use sgx_tstd::sync::SgxMutexGuard;
use substratee_worker_primitives::get_account;

pub fn get_dummy_map(storage: &mut SgxMutexGuard<AccountsNonceStorage>) {
    let main_account_one: AccountId = get_account("first_account");
    let main_account_two: AccountId = get_account("second_account");
    let main_account_three: AccountId = get_account("third_account");
    let dummy_account_one: AccountId = get_account("first_dummy_account");
    let dummy_account_two: AccountId = get_account("second_dummy_account");
    let dummy_account_three: AccountId = get_account("third_dummy_account");

    storage
        .accounts_storage
        .accounts
        .insert(main_account_one.encode(), vec![dummy_account_one.clone()]);
    storage.accounts_storage.accounts.insert(
        main_account_two.encode(),
        vec![dummy_account_one.clone(), dummy_account_two.clone()],
    );
    storage.accounts_storage.accounts.insert(
        main_account_three.encode(),
        vec![dummy_account_one, dummy_account_two, dummy_account_three],
    );
}

pub fn initialize_dummy() {
    accounts_nonce_storage::create_in_memory_accounts_and_nonce_storage(vec![]);
    let mutex = accounts_nonce_storage::load_registry().unwrap();
    let mut storage = mutex.lock().unwrap();
    get_dummy_map(&mut storage);
}

#[allow(unused)]
pub fn test_check_if_main_account_registered() {
    initialize_dummy();
    let account_to_find_real: AccountId = get_account("first_account");
    let account_to_find_false: AccountId = get_account("false_account");
    assert!(
        accounts_nonce_storage::check_if_main_account_registered(account_to_find_real).unwrap(),
    );
    assert!(
        !accounts_nonce_storage::check_if_main_account_registered(account_to_find_false).unwrap(),
    );
}

#[allow(unused)]
pub fn test_check_if_proxy_registered() {
    let main_account: AccountId = get_account("first_account");
    let main_account_false: AccountId = get_account("false_account");
    let dummy_account_one: AccountId = get_account("first_dummy_account");
    let dummy_account_false: AccountId = get_account("false_dummy_account");
    assert!(accounts_nonce_storage::check_if_proxy_registered(
        main_account.clone(),
        dummy_account_one
    )
    .unwrap(),);
    assert!(!accounts_nonce_storage::check_if_proxy_registered(
        main_account,
        dummy_account_false.clone()
    )
    .unwrap(),);
    assert_eq!(
        accounts_nonce_storage::check_if_proxy_registered(main_account_false, dummy_account_false),
        Err(Error::ProxyNotRegistered)
    );
}

#[allow(unused)]
pub fn test_add_main_account() {
    let main_account: AccountId = get_account("new_account");
    accounts_nonce_storage::add_main_account(main_account.clone());
    assert!(accounts_nonce_storage::check_if_main_account_registered(main_account).unwrap(),);
}

#[allow(unused)]
pub fn test_remove_main_account() {
    let main_account: AccountId = get_account("first_account");
    assert!(
        accounts_nonce_storage::check_if_main_account_registered(main_account.clone()).unwrap(),
    );
    accounts_nonce_storage::remove_main_account(main_account.clone());
    assert!(!accounts_nonce_storage::check_if_main_account_registered(main_account).unwrap(),);
}

#[allow(unused)]
pub fn test_add_proxy_account() {
    let main_account: AccountId = get_account("first_account");
    let new_proxy_account: AccountId = get_account("new_account");
    accounts_nonce_storage::add_proxy(main_account.clone(), new_proxy_account.clone());
    assert!(
        accounts_nonce_storage::check_if_proxy_registered(main_account, new_proxy_account).unwrap()
    );
}

#[allow(unused)]
pub fn test_remove_proxy_account() {
    let main_account: AccountId = get_account("first_account");
    let dummy_account_one: AccountId = get_account("first_dummy_account");
    assert!(accounts_nonce_storage::check_if_proxy_registered(
        main_account.clone(),
        dummy_account_one.clone()
    )
    .unwrap());
    accounts_nonce_storage::remove_proxy(main_account.clone(), dummy_account_one.clone());
    assert!(
        !accounts_nonce_storage::check_if_proxy_registered(main_account, dummy_account_one)
            .unwrap(),
    );
}
