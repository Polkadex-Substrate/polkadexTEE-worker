use sgx_types::{sgx_status_t, SgxResult};
use crate::polkadex::PolkadexAccountsStorage;
use sp_std::prelude::*;
use sgx_tstd::collections::HashMap;
use codec::Encode;
use std::{
    sync::atomic::{AtomicPtr, Ordering},
    sync::{Arc, SgxMutex},
};
use sgx_rand::{Rng, SeedableRng, StdRng};
use crate::polkadex;
use sp_core::blake2_256;
static GLOBAL_ACCOUNTS_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn get_dummy_map() -> HashMap<[u8;32],Vec<[u8;32]>>{
    let mut hashmap: HashMap<[u8;32],Vec<[u8;32]>> = HashMap::new();
    let vec = vec![2,3,4];

    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let main_account_one:[u8;32]  = rng.gen();

    let main_account_two:[u8;32]  = Vec::from("second_account").using_encoded(blake2_256);
    let main_account_three:[u8;32]  = Vec::from("third_account").using_encoded(blake2_256);
    let dummy_account_one:[u8;32]  = Vec::from("first_dummy_account").using_encoded(blake2_256);
    let dummy_account_two:[u8;32]  = Vec::from("second_dummy_account").using_encoded(blake2_256);
    let dummy_account_three:[u8;32]  = Vec::from("third_dummy_account").using_encoded(blake2_256);
    hashmap.insert(main_account_one, vec![dummy_account_one]);
    hashmap.insert(main_account_two, vec![dummy_account_one, dummy_account_two]);
    hashmap.insert(main_account_three, vec![dummy_account_one, dummy_account_two, dummy_account_three]);
    hashmap
}

pub fn pointer_initialize() -> sgx_status_t{
    let mut hashmap: HashMap<[u8;32],Vec<[u8;32]>> = get_dummy_map();
    let polkadex_hashmap: PolkadexAccountsStorage = PolkadexAccountsStorage::from_hashmap(hashmap);
    let storage_ptr = Arc::new(SgxMutex::<PolkadexAccountsStorage>::new(polkadex_hashmap));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ACCOUNTS_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    sgx_status_t::SGX_SUCCESS
}

pub fn load_proxy_registry() -> SgxResult<&'static SgxMutex<PolkadexAccountsStorage>> {
    let ptr =
        GLOBAL_ACCOUNTS_STORAGE.load(Ordering::SeqCst) as *mut SgxMutex<PolkadexAccountsStorage>;
    if ptr.is_null() {
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}

#[allow(unused)]
pub fn test_check_main_account() {
    pointer_initialize();
    let account_to_find_real:[u8;32]  = Vec::from("first_account").using_encoded(blake2_256);
    let account_to_find_false:[u8;32]  = Vec::from("false_account").using_encoded(blake2_256);
    assert_eq!(polkadex::check_main_account(account_to_find_real), Ok(true));
    assert_eq!(polkadex::check_main_account(account_to_find_false), Ok(false));
}

#[allow(unused)]
pub fn test_check_proxy_account() {
    pointer_initialize();
    let main_account:[u8;32]  = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_false:[u8;32]  = Vec::from("false_account").using_encoded(blake2_256);
    let dummy_account_one:[u8;32]  = Vec::from("first_dummy_account").using_encoded(blake2_256);
    let dummy_account_false:[u8;32]  = Vec::from("false_dummy_account").using_encoded(blake2_256);
    assert_eq!(polkadex::check_proxy_account(main_account, dummy_account_one), Ok(true));
    assert_eq!(polkadex::check_proxy_account(main_account, dummy_account_false), Ok(false));
    assert_eq!(polkadex::check_proxy_account(main_account_false, dummy_account_false), Ok(false));
}

#[allow(unsued)]
pub fn test_add_main_account() {
    pointer_initialize();
    let main_account:[u8;32]  = Vec::from("new_account").using_encoded(blake2_256);
    polkadex::add_main_account(main_account);
    let mutex = load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    assert_eq!(proxy_storage.accounts.contains_key(&main_account), true);
}

#[allow(unsued)]
pub fn test_remove_main_account() {
    pointer_initialize();
    let main_account:[u8;32]  = Vec::from("first_account").using_encoded(blake2_256);
    polkadex::remove_main_account(main_account);
    let mutex = load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    assert_eq!(proxy_storage.accounts.contains_key(&main_account), false);
}

#[allow(unsued)]
pub fn test_add_proxy_account() {
    pointer_initialize();
    let main_account:[u8;32]  = Vec::from("first_account").using_encoded(blake2_256);
    let new_proxy_account:[u8;32]  = Vec::from("new_account").using_encoded(blake2_256);
    polkadex::add_proxy(main_account, new_proxy_account);
    let mutex = load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    let proxies = proxy_storage.accounts.get(&main_account).unwrap();
    assert_eq!(proxies.contains(&new_proxy_account), true);
}

#[allow(unsued)]
pub fn test_remove_proxy_account() {
    pointer_initialize();
    let main_account:[u8;32]  = Vec::from("first_account").encode().using_encoded(blake2_256);
    let dummy_account_one:[u8;32]  = Vec::from("first_dummy_account").using_encoded(blake2_256);
    polkadex::remove_proxy(main_account, dummy_account_one);
    let mutex = load_proxy_registry().unwrap();
    let mut proxy_storage = mutex.lock().unwrap();
    let proxies = proxy_storage.accounts.get(&main_account).unwrap();
    assert_eq!(proxies.contains(&dummy_account_one), false);
}


