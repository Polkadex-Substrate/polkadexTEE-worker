use codec::Encode;
use polkadex_sgx_primitives::{accounts::get_account, AccountId};
use sgx_rand::{Rng, SeedableRng, StdRng};
use sgx_tstd::collections::HashMap;
use sgx_tstd::format;
use sgx_tstd::sync::SgxMutexGuard;
use sgx_types::{sgx_status_t, SgxResult};
use std::{
    sync::atomic::{AtomicPtr, Ordering},
    sync::{Arc, SgxMutex},
};

use crate::polkadex;
use crate::polkadex::EncodedAccountId;
use crate::polkadex::PolkadexAccountsStorage;

pub fn get_dummy_map(storage: &mut SgxMutexGuard<PolkadexAccountsStorage>) {
    let main_account_one: AccountId = get_account("first_account");
    let main_account_two: AccountId = get_account("second_account");
    let main_account_three: AccountId = get_account("third_account");
    let dummy_account_one: AccountId = get_account("first_dummy_account");
    let dummy_account_two: AccountId = get_account("second_dummy_account");
    let dummy_account_three: AccountId = get_account("third_dummy_account");

    storage
        .accounts
        .insert(main_account_one.encode(), vec![dummy_account_one.clone()]);
    storage.accounts.insert(
        main_account_two.encode(),
        vec![dummy_account_one.clone(), dummy_account_two.clone()],
    );
    storage.accounts.insert(
        main_account_three.encode(),
        vec![dummy_account_one, dummy_account_two, dummy_account_three],
    );
}

pub fn initialize_dummy() {
    polkadex::create_in_memory_account_storage(vec![]);
    let mutex = polkadex::load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    get_dummy_map(&mut proxy_storage);
}

#[allow(unused)]
pub fn test_check_if_main_account_registered() {
    initialize_dummy();
    let account_to_find_real: AccountId = get_account("first_account");
    let account_to_find_false: AccountId = get_account("false_account");
    assert_eq!(
        polkadex::check_if_main_account_registered(account_to_find_real),
        Ok(true)
    );
    assert_eq!(
        polkadex::check_if_main_account_registered(account_to_find_false),
        Ok(false)
    );
}

#[allow(unused)]
pub fn test_check_if_proxy_registered() {
    let main_account: AccountId = get_account("first_account");
    let main_account_false: AccountId = get_account("false_account");
    let dummy_account_one: AccountId = get_account("first_dummy_account");
    let dummy_account_false: AccountId = get_account("false_dummy_account");
    assert_eq!(
        polkadex::check_if_proxy_registered(main_account.clone(), dummy_account_one),
        Ok(true)
    );
    assert_eq!(
        polkadex::check_if_proxy_registered(main_account, dummy_account_false.clone()),
        Ok(false)
    );
    assert_eq!(
        polkadex::check_if_proxy_registered(main_account_false, dummy_account_false),
        Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
    );
}

#[allow(unused)]
pub fn test_add_main_account() {
    let main_account: AccountId = get_account("new_account");
    polkadex::add_main_account(main_account.clone());
    assert_eq!(
        polkadex::check_if_main_account_registered(main_account),
        Ok(true)
    );
}

#[allow(unused)]
pub fn test_remove_main_account() {
    let main_account: AccountId = get_account("first_account");
    assert_eq!(
        polkadex::check_if_main_account_registered(main_account.clone()),
        Ok(true)
    );
    polkadex::remove_main_account(main_account.clone());
    assert_eq!(
        polkadex::check_if_main_account_registered(main_account),
        Ok(false)
    );
}

#[allow(unused)]
pub fn test_add_proxy_account() {
    let main_account: AccountId = get_account("first_account");
    let new_proxy_account: AccountId = get_account("new_account");
    polkadex::add_proxy(main_account.clone(), new_proxy_account.clone());
    assert_eq!(
        polkadex::check_if_proxy_registered(main_account, new_proxy_account),
        Ok(true)
    );
}

#[allow(unused)]
pub fn test_remove_proxy_account() {
    let main_account: AccountId = get_account("first_account");
    let dummy_account_one: AccountId = get_account("first_dummy_account");
    assert_eq!(
        polkadex::check_if_proxy_registered(main_account.clone(), dummy_account_one.clone()),
        Ok(true)
    );
    polkadex::remove_proxy(main_account.clone(), dummy_account_one.clone());
    assert_eq!(
        polkadex::check_if_proxy_registered(main_account, dummy_account_one),
        Ok(false)
    );
}
