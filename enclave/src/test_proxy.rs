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

pub fn get_dummy_map(storage: &mut SgxMutexGuard<PolkadexAccountsStorage>) {
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_two: [u8; 32] = Vec::from("second_account").using_encoded(blake2_256);
    let main_account_three: [u8; 32] = Vec::from("third_account").using_encoded(blake2_256);
    let dummy_account_one: [u8; 32] = Vec::from("first_dummy_account").using_encoded(blake2_256);
    let dummy_account_two: [u8; 32] = Vec::from("second_dummy_account").using_encoded(blake2_256);
    let dummy_account_three: [u8; 32] = Vec::from("third_dummy_account").using_encoded(blake2_256);

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
    let account_to_find_real: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let account_to_find_false: [u8; 32] = Vec::from("false_account").using_encoded(blake2_256);
    assert_eq!(polkadex::check_main_account(account_to_find_real), Ok(true));
    assert_eq!(
        polkadex::check_main_account(account_to_find_false),
        Ok(false)
    );
}

#[allow(unused)]
pub fn test_check_proxy_account() {
    let main_account: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_false: [u8; 32] = Vec::from("false_account").using_encoded(blake2_256);
    let dummy_account_one: [u8; 32] = Vec::from("first_dummy_account").using_encoded(blake2_256);
    let dummy_account_false: [u8; 32] = Vec::from("false_dummy_account").using_encoded(blake2_256);
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
    let main_account: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
    polkadex::add_main_account(main_account);
    let mutex = polkadex::load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    assert_eq!(proxy_storage.accounts.contains_key(&main_account), true);
}

#[allow(unsued)]
pub fn test_remove_main_account() {
    let main_account: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let mutex = polkadex::load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    assert_eq!(proxy_storage.accounts.contains_key(&main_account), true);
    polkadex::remove_main_account(main_account);
    assert_eq!(proxy_storage.accounts.contains_key(&main_account), false);
}

#[allow(unsued)]
pub fn test_add_proxy_account() {
    let main_account: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let new_proxy_account: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
    polkadex::add_proxy(main_account, new_proxy_account);
    let mutex = polkadex::load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    let proxies = proxy_storage.accounts.get(&main_account).unwrap();
    assert_eq!(proxies.contains(&new_proxy_account), true);
}

#[allow(unsued)]
pub fn test_remove_proxy_account() {
    let main_account: [u8; 32] = Vec::from("first_account")
        .encode()
        .using_encoded(blake2_256);
    let dummy_account_one: [u8; 32] = Vec::from("first_dummy_account").using_encoded(blake2_256);
    let mutex = polkadex::load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    let proxies = proxy_storage.accounts.get(&main_account).unwrap();
    assert_eq!(proxies.contains(&dummy_account_one), true);
    polkadex::remove_proxy(main_account, dummy_account_one);
    let proxies = proxy_storage.accounts.get(&main_account).unwrap();
    assert_eq!(proxies.contains(&dummy_account_one), false);
}
