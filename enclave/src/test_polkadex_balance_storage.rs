/// Tests for Polkadex Balance Storage
///
///
use crate::polkadex_balance_storage::*;
use sgx_rand;
use substratee_node_primitives::AssetId;

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
