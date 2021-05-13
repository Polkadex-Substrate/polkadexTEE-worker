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

// #[allow(unused)]
// pub fn test_create_balance_storage() {
//     assert_eq!(create_in_memory_balance_storage().is_ok(), true);
//     assert_eq!(load_balance_storage().is_ok(), true);
// }
//
// #[allow(unused)]
// pub fn test_balance_struct() {
//     let main_acc: [u8; 32] = sgx_rand::random();
//     let non_registered_acc: [u8; 32] = sgx_rand::random();
//     let mut balances = PolkadexBalanceStorage::create();
//
//     // Set Free balance
//     assert_eq!(
//         balances
//             .set_free_balance(AssetId::POLKADEX, main_acc, 100u128)
//             .is_ok(),
//         true
//     );
//     // Set Reserved balance
//     assert_eq!(
//         balances
//             .set_free_balance(AssetId::POLKADEX, main_acc, 200u128)
//             .is_ok(),
//         true
//     );
//     // Read balance
//     assert_eq!(
//         balances
//             .read_balance(AssetId::POLKADEX, main_acc)
//             .unwrap()
//             .0,
//         100u128
//     );
//     assert_eq!(
//         balances
//             .read_balance(AssetId::POLKADEX, main_acc)
//             .unwrap()
//             .1,
//         200u128
//     );
//     // Deposit Balance
//     assert_eq!(
//         balances
//             .deposit(AssetId::POLKADEX, main_acc, 100u128)
//             .is_ok(),
//         true
//     );
//     assert_eq!(
//         balances
//             .read_balance(AssetId::POLKADEX, main_acc)
//             .unwrap()
//             .0,
//         200u128
//     );
//     // Withdraw Balance
//     assert_eq!(
//         balances
//             .withdraw(AssetId::POLKADEX, main_acc, 100u128)
//             .is_ok(),
//         true
//     );
//     assert_eq!(
//         balances
//             .read_balance(AssetId::POLKADEX, main_acc)
//             .unwrap()
//             .0,
//         100u128
//     );
//     // Test Withdraw Underflow
//     assert_eq!(
//         balances
//             .withdraw(AssetId::POLKADEX, main_acc, u128::MAX)
//             .is_ok(),
//         true
//     );
//     assert_eq!(
//         balances
//             .read_balance(AssetId::POLKADEX, main_acc)
//             .unwrap()
//             .0,
//         0u128
//     );
//     // Test Deposit Overflow
//     assert_eq!(
//         balances
//             .deposit(AssetId::POLKADEX, main_acc, u128::MAX)
//             .is_ok(),
//         true
//     );
//     assert_eq!(
//         balances
//             .read_balance(AssetId::POLKADEX, main_acc)
//             .unwrap()
//             .0,
//         u128::MAX
//     );
//
//     // Test Non Registered Account
//     assert_eq!(
//         balances
//             .read_balance(AssetId::POLKADEX, non_registered_acc)
//             .is_some(),
//         false
//     );
//     assert_eq!(
//         balances
//             .set_free_balance(AssetId::POLKADEX, non_registered_acc, u128::MAX)
//             .is_ok(),
//         false
//     );
//     assert_eq!(
//         balances
//             .set_reserve_balance(AssetId::POLKADEX, non_registered_acc, u128::MAX)
//             .is_ok(),
//         false
//     );
//     assert_eq!(
//         balances
//             .deposit(AssetId::POLKADEX, non_registered_acc, u128::MAX)
//             .is_ok(),
//         false
//     );
//     assert_eq!(
//         balances
//             .withdraw(AssetId::POLKADEX, non_registered_acc, u128::MAX)
//             .is_ok(),
//         false
//     );
// }

#[allow(unused)]
pub fn dummy_map(balance_storage: &mut SgxMutexGuard<PolkadexBalanceStorage>) {
    let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
    let main_account_two: [u8; 32] = Vec::from("second_account").using_encoded(blake2_256);
    let key_one = PolkadexBalanceKey::from(AssetId::POLKADEX, main_account_one);
    let value_one = Balances::from(100u128, 0u128);
    let key_two = PolkadexBalanceKey::from(AssetId::POLKADEX, main_account_two);
    let value_two = Balances::from(100u128, 0u128);
    balance_storage.storage.insert(key_one, value_one);
    balance_storage.storage.insert(key_two, value_two);
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

    //let balance = get_balances(main_account_one, AssetId::POLKADEX);
    //assert_eq!(balance, Ok((150u128, 0u128)))
}

// #[allow(unused)]
// pub fn test_withdraw() {
//     pointer_initialize();
//     let main_account_one: [u8; 32] = Vec::from("first_account").using_encoded(blake2_256);
//     assert_eq!(
//         withdraw(main_account_one, AssetId::POLKADEX, 50u128),
//         Ok(())
//     );
//     let mutex = load_proxy_registry().unwrap();
//     let mut balance_storage = mutex.lock().unwrap();
//     let balance = balance_storage
//         .storage
//         .get(&(AssetId::POLKADEX, main_account_one))
//         .cloned()
//         .unwrap();
//     assert_eq!(balance.0, 50u128);
//
//     //Test Error
//     //assert_noop!(withdraw(main_account_one, AssetId::POLKADEX, 200u128), sgx_status_t::SGX_ERROR_UNEXPECTED);
// }
//
// //Test PolkadexBalanceStorage implemented Methods
// #[allow(unused)]
// pub fn test_set_free_balance() {
//     let mut polkadex_balance_storage = PolkadexBalanceStorage::create();
//     let new_account_one: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
//     assert_eq!(
//         polkadex_balance_storage.set_free_balance(AssetId::POLKADEX, new_account_one, 100u128),
//         Ok(())
//     );
//     let balance = polkadex_balance_storage
//         .storage
//         .get(&(AssetId::POLKADEX, new_account_one))
//         .cloned()
//         .unwrap();
//     assert_eq!(balance.0, 100u128);
// }
//
// #[allow(unused)]
// pub fn test_set_reserve_balance() {
//     let mut polkadex_balance_storage = PolkadexBalanceStorage::create();
//     let new_account_one: [u8; 32] = Vec::from("new_account").using_encoded(blake2_256);
//     assert_eq!(
//         polkadex_balance_storage.set_reserve_balance(AssetId::POLKADEX, new_account_one, 100u128),
//         Ok(())
//     );
//     let balance = polkadex_balance_storage
//         .storage
//         .get(&(AssetId::POLKADEX, new_account_one))
//         .cloned()
//         .unwrap();
//     assert_eq!(balance.1, 100u128);
// }
