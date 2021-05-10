use crate::polkadex;
use crate::polkadex::PolkadexAccountsStorage;
/// Tests for Polkadex Balance Storage
///
///
use crate::polkadex_balance_storage::*;
use codec::Encode;
use sgx_rand::{Rng, SeedableRng, StdRng};
use sgx_tstd::collections::HashMap;
use sgx_types::{sgx_status_t, SgxResult};
use sp_core::blake2_256;
use sp_std::prelude::*;
use std::{
    sync::atomic::{AtomicPtr, Ordering},
    sync::{Arc, SgxMutex},
};
use substratee_node_primitives::AssetId;

static GLOBAL_ACCOUNTS_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[allow(unused)]
pub fn test_create_balance_storage() {
    assert_eq!(create_in_memory_balance_storage().is_ok(), true);
    assert_eq!(load_balance_storage().is_ok(), true);
}

#[allow(unused)]
pub fn test_balance_struct() {
    let main_acc: [u8; 32] = sgx_rand::random();
    let non_registered_acc: [u8; 32] = sgx_rand::random();
    let mut balances = PolkadexBalanceStorage::create();

    // Set Free balance
    assert_eq!(
        balances
            .set_free_balance(AssetId::POLKADEX, main_acc, 100u128)
            .is_ok(),
        true
    );
    // Set Reserved balance
    assert_eq!(
        balances
            .set_free_balance(AssetId::POLKADEX, main_acc, 200u128)
            .is_ok(),
        true
    );
    // Read balance
    assert_eq!(
        balances
            .read_balance(AssetId::POLKADEX, main_acc)
            .unwrap()
            .0,
        100u128
    );
    assert_eq!(
        balances
            .read_balance(AssetId::POLKADEX, main_acc)
            .unwrap()
            .1,
        200u128
    );
    // Deposit Balance
    assert_eq!(
        balances
            .deposit(AssetId::POLKADEX, main_acc, 100u128)
            .is_ok(),
        true
    );
    assert_eq!(
        balances
            .read_balance(AssetId::POLKADEX, main_acc)
            .unwrap()
            .0,
        200u128
    );
    // Withdraw Balance
    assert_eq!(
        balances
            .withdraw(AssetId::POLKADEX, main_acc, 100u128)
            .is_ok(),
        true
    );
    assert_eq!(
        balances
            .read_balance(AssetId::POLKADEX, main_acc)
            .unwrap()
            .0,
        100u128
    );
    // Test Withdraw Underflow
    assert_eq!(
        balances
            .withdraw(AssetId::POLKADEX, main_acc, u128::MAX)
            .is_ok(),
        true
    );
    assert_eq!(
        balances
            .read_balance(AssetId::POLKADEX, main_acc)
            .unwrap()
            .0,
        0u128
    );
    // Test Deposit Overflow
    assert_eq!(
        balances
            .deposit(AssetId::POLKADEX, main_acc, u128::MAX)
            .is_ok(),
        true
    );
    assert_eq!(
        balances
            .read_balance(AssetId::POLKADEX, main_acc)
            .unwrap()
            .0,
        u128::MAX
    );

    // Test Non Registered Account
    assert_eq!(
        balances
            .read_balance(AssetId::POLKADEX, non_registered_acc)
            .is_some(),
        false
    );
    assert_eq!(
        balances
            .set_free_balance(AssetId::POLKADEX, non_registered_acc, u128::MAX)
            .is_ok(),
        false
    );
    assert_eq!(
        balances
            .set_reserve_balance(AssetId::POLKADEX, non_registered_acc, u128::MAX)
            .is_ok(),
        false
    );
    assert_eq!(
        balances
            .deposit(AssetId::POLKADEX, non_registered_acc, u128::MAX)
            .is_ok(),
        false
    );
    assert_eq!(
        balances
            .withdraw(AssetId::POLKADEX, non_registered_acc, u128::MAX)
            .is_ok(),
        false
    );
}

pub fn dummy_map() -> HashMap<(AssetId, [u8; 32]), (u128, u128)> {
    let mut hashmap: HashMap<(AssetId, [u8; 32]), (u128, u128)> = HashMap::new();
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_two: [u8; 32] = Vec::from("second_account").using_encoded(blake2_256);
    hashmap.insert((AssetId::POLKADEX, main_account_one), (100u128, 0u128));
    hashmap.insert((AssetId::POLKADEX, main_account_two), (200u128, 0u128));
    hashmap
}

pub fn pointer_initialize() -> sgx_status_t {
    let mut hashmap: HashMap<(AssetId, [u8; 32]), (u128, u128)> = dummy_map();
    let polkadex_hashmap: PolkadexBalanceStorage = PolkadexBalanceStorage::from_hashmap(hashmap);
    let storage_ptr = Arc::new(SgxMutex::<PolkadexBalanceStorage>::new(polkadex_hashmap));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ACCOUNTS_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    sgx_status_t::SGX_SUCCESS
}

pub fn load_proxy_registry() -> SgxResult<&'static SgxMutex<PolkadexBalanceStorage>> {
    let ptr =
        GLOBAL_ACCOUNTS_STORAGE.load(Ordering::SeqCst) as *mut SgxMutex<PolkadexBalanceStorage>;
    if ptr.is_null() {
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    } else {
        Ok(unsafe { &*ptr })
    }
}

#[allow(unused)]
pub fn test_deposit() {
    pointer_initialize();
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    assert_eq!(deposit(main_account_one, AssetId::POLKADEX, 50u128), Ok(()));
    let mutex = load_proxy_registry().unwrap();
    let mut balance_storage = mutex.lock().unwrap();
    let balance = balance_storage
        .storage
        .get(&(AssetId::POLKADEX, main_account_one))
        .cloned()
        .unwrap();
    assert_eq!(balance.0, 150u128);
}

#[allow(unused)]
pub fn test_withdraw() {
    pointer_initialize();
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    assert_eq!(
        withdraw(main_account_one, AssetId::POLKADEX, 50u128),
        Ok(())
    );
    let mutex = load_proxy_registry().unwrap();
    let mut balance_storage = mutex.lock().unwrap();
    let balance = balance_storage
        .storage
        .get(&(AssetId::POLKADEX, main_account_one))
        .cloned()
        .unwrap();
    assert_eq!(balance.0, 50u128);

    //Test Error
    //assert_noop!(withdraw(main_account_one, AssetId::POLKADEX, 200u128), sgx_status_t::SGX_ERROR_UNEXPECTED);
}

//Test PolkadexBalanceStorage implemented Methods
#[allow(unused)]
pub fn test_set_free_balance() {
    let mut polkadex_balance_storage = PolkadexBalanceStorage::create();
    let new_account_one: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
    assert_eq!(
        polkadex_balance_storage.set_free_balance(AssetId::POLKADEX, new_account_one, 100u128),
        Ok(())
    );
    let balance = polkadex_balance_storage
        .storage
        .get(&(AssetId::POLKADEX, new_account_one))
        .cloned()
        .unwrap();
    assert_eq!(balance.0, 100u128);
}

#[allow(unused)]
pub fn test_set_reserve_balance() {
    let mut polkadex_balance_storage = PolkadexBalanceStorage::create();
    let new_account_one: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
    assert_eq!(
        polkadex_balance_storage.set_reserve_balance(AssetId::POLKADEX, new_account_one, 100u128),
        Ok(())
    );
    let balance = polkadex_balance_storage
        .storage
        .get(&(AssetId::POLKADEX, new_account_one))
        .cloned()
        .unwrap();
    assert_eq!(balance.1, 100u128);
}
