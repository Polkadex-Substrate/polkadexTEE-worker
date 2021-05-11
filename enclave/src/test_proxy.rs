use crate::polkadex;
use crate::polkadex::PolkadexAccountsStorage;
use codec::Encode;
use sgx_rand::{Rng, SeedableRng, StdRng};
use sgx_tstd::collections::HashMap;
use sgx_tstd::sync::SgxMutexGuard;
use sgx_types::{sgx_status_t, SgxResult};
use sp_core::blake2_256;
use sp_std::prelude::*;
use std::{
    sync::atomic::{AtomicPtr, Ordering},
    sync::{Arc, SgxMutex},
};
use polkadex_primitives::AccountId;

pub fn get_dummy_map(storage: &mut SgxMutexGuard<PolkadexAccountsStorage>) {
    let main_account_one: AccountId = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_two: AccountId = Vec::from("second_account").using_encoded(blake2_256);
    let main_account_three: AccountId = Vec::from("third_account").using_encoded(blake2_256);
    let dummy_account_one: AccountId = Vec::from("first_dummy_account").using_encoded(blake2_256);
    let dummy_account_two: AccountId = Vec::from("second_dummy_account").using_encoded(blake2_256);
    let dummy_account_three: AccountId = Vec::from("third_dummy_account").using_encoded(blake2_256);

    storage
        .accounts
        .insert(main_account_one, vec![dummy_account_one]);
    storage
        .accounts
        .insert(main_account_two, vec![dummy_account_one, dummy_account_two]);
    storage.accounts.insert(
        main_account_three,
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
pub fn test_check_main_account() {
    initialize_dummy();
    let account_to_find_real: AccountId = Vec::from("first_account").using_encoded(blake2_256);
    let account_to_find_false: AccountId = Vec::from("false_account").using_encoded(blake2_256);
    assert_eq!(polkadex::check_main_account(account_to_find_real), Ok(true));
    assert_eq!(
        polkadex::check_main_account(account_to_find_false),
        Ok(false)
    );
}

#[allow(unused)]
pub fn test_check_proxy_account() {
    let main_account: AccountId = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_false: AccountId = Vec::from("false_account").using_encoded(blake2_256);
    let dummy_account_one: AccountId = Vec::from("first_dummy_account").using_encoded(blake2_256);
    let dummy_account_false: AccountId = Vec::from("false_dummy_account").using_encoded(blake2_256);
    assert_eq!(
        polkadex::check_proxy_account(main_account, dummy_account_one),
        Ok(true)
    );
    assert_eq!(
        polkadex::check_proxy_account(main_account, dummy_account_false),
        Ok(false)
    );
    assert_eq!(
        polkadex::check_proxy_account(main_account_false, dummy_account_false),
        Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
    );
}

#[allow(unsued)]
pub fn test_add_main_account() {
    let main_account: AccountId = Vec::from("new_account").using_encoded(blake2_256);
    polkadex::add_main_account(main_account);
    assert_eq!(polkadex::check_main_account(main_account), Ok(true));
}

#[allow(unsued)]
pub fn test_remove_main_account() {
    let main_account: AccountId = Vec::from("first_account").using_encoded(blake2_256);
    assert_eq!(polkadex::check_main_account(main_account), Ok(true));
    polkadex::remove_main_account(main_account);
    assert_eq!(polkadex::check_main_account(main_account), Ok(false));
}

#[allow(unsued)]
pub fn test_add_proxy_account() {
    let main_account: AccountId = Vec::from("first_account").using_encoded(blake2_256);
    let new_proxy_account: AccountId = Vec::from("new_account").using_encoded(blake2_256);
    polkadex::add_proxy(main_account, new_proxy_account);
    assert_eq!(
        polkadex::check_proxy_account(main_account, new_proxy_account),
        Ok(true)
    );
}

#[allow(unsued)]
pub fn test_remove_proxy_account() {
    let main_account: AccountId = Vec::from("first_account").using_encoded(blake2_256);
    let dummy_account_one: AccountId = Vec::from("first_dummy_account").using_encoded(blake2_256);
    assert_eq!(
        polkadex::check_proxy_account(main_account, dummy_account_one),
        Ok(true)
    );
    polkadex::remove_proxy(main_account, dummy_account_one);
    assert_eq!(
        polkadex::check_proxy_account(main_account, dummy_account_one),
        Ok(false)
    );
}
