use crate::polkadex;
use crate::polkadex::PolkadexAccountsStorage;
use crate::polkadex_balance_storage::*;
use codec::Encode;
use log::*;
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
use substratee_node_primitives::AssetId;

#[allow(unused)]
pub fn dummy_map(balance_storage: &mut SgxMutexGuard<PolkadexBalanceStorage>) {
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_two: [u8; 32] = Vec::from("second_account").using_encoded(blake2_256);
    let key_one = PolkadexBalanceKey::from(AssetId::POLKADEX, main_account_one);
    let value_one = Balances::from(100u128, 0u128);
    let key_two = PolkadexBalanceKey::from(AssetId::POLKADEX, main_account_two);
    let value_two = Balances::from(100u128, 0u128);
    balance_storage.storage.insert(key_one.encode(), value_one);
    balance_storage.storage.insert(key_two.encode(), value_two);
}

#[allow(unused)]
pub fn initialize_dummy() {
    {
        create_in_memory_balance_storage();
    }
    let mutex = load_balance_storage().unwrap();
    let mut balance_storage = mutex.lock().unwrap();
    dummy_map(&mut balance_storage);
}

#[allow(unused)]
pub fn test_deposit() {
    {
        initialize_dummy();
    }
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);

    deposit(main_account_one, AssetId::POLKADEX, 50u128);

    let balance = get_balances(main_account_one, AssetId::POLKADEX);
    assert_eq!(balance, Ok(Balances::from(150u128, 0u128)))
}

#[allow(unused)]
pub fn test_withdraw() {
    initialize_dummy();
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    assert_eq!(
        withdraw(main_account_one, AssetId::POLKADEX, 50u128),
        Ok(())
    );
    let balance = get_balances(main_account_one, AssetId::POLKADEX);
    assert_eq!(balance, Ok(Balances::from(50u128, 0u128)));

    //Test Error
    assert_eq!(
        withdraw(main_account_one, AssetId::POLKADEX, 200u128),
        Err(sgx_status_t::SGX_ERROR_UNEXPECTED)
    );
}

//Test PolkadexBalanceStorage implemented Methods
#[allow(unused)]
pub fn test_set_free_balance() {
    let new_account_one: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
    let key_new = PolkadexBalanceKey::from(AssetId::POLKADEX, new_account_one);
    let mut polkadex_balance_storage = PolkadexBalanceStorage::create();
    polkadex_balance_storage
        .storage
        .insert(key_new.encode(), Balances::from(0u128, 0u128));
    assert_eq!(
        polkadex_balance_storage.set_free_balance(AssetId::POLKADEX, new_account_one, 100u128),
        Ok(())
    );
    let balance = polkadex_balance_storage
        .storage
        .get(&key_new.encode())
        .cloned()
        .unwrap();
    assert_eq!(balance.free, 100u128);
}

#[allow(unused)]
pub fn test_set_reserve_balance() {
    let new_account_one: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
    let key_new = PolkadexBalanceKey::from(AssetId::POLKADEX, new_account_one);
    let mut polkadex_balance_storage = PolkadexBalanceStorage::create();
    polkadex_balance_storage
        .storage
        .insert(key_new.encode(), Balances::from(0u128, 0u128));
    assert_eq!(
        polkadex_balance_storage.set_reserve_balance(AssetId::POLKADEX, new_account_one, 100u128),
        Ok(())
    );
    let balance = polkadex_balance_storage
        .storage
        .get(&key_new.encode())
        .cloned()
        .unwrap();
    assert_eq!(balance.reserved, 100u128);
}
