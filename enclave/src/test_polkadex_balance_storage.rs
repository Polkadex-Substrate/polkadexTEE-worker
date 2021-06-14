use codec::Encode;
use sgx_tstd::sync::SgxMutexGuard;
use sgx_types::sgx_status_t;
use sp_std::prelude::*;

use polkadex_sgx_primitives::accounts::get_account;
use polkadex_sgx_primitives::{AccountId, AssetId};

use crate::polkadex_balance_storage::*;
use crate::polkadex_gateway::GatewayError;

#[allow(unused)]
pub fn dummy_map(balance_storage: &mut SgxMutexGuard<PolkadexBalanceStorage>) {
    let main_account_one: AccountId = get_account("first_account");
    let main_account_two: AccountId = get_account("second_account");
    let key_one = PolkadexBalanceKey::from(AssetId::POLKADEX, main_account_one);
    let value_one = Balances::from(100u128, 0u128);
    let key_two = PolkadexBalanceKey::from(AssetId::POLKADEX, main_account_two);
    let value_two = Balances::from(100u128, 0u128);
    balance_storage.storage.insert(key_one.encode(), value_one);
    balance_storage.storage.insert(key_two.encode(), value_two);
}

#[allow(unused)]
pub fn initialize_dummy() {
    create_in_memory_balance_storage();
    let mutex = load_balance_storage().unwrap();
    let mut balance_storage = mutex.lock().unwrap();
    dummy_map(&mut balance_storage);
}

#[allow(unused)]
pub fn test_deposit() {
    initialize_dummy();
    let main_account_one: AccountId = get_account("first_account");
    lock_storage_and_deposit(main_account_one.clone(), AssetId::POLKADEX, 50u128);
    let balance = lock_storage_and_get_balances(main_account_one, AssetId::POLKADEX);
    assert_eq!(balance, Ok(Balances::from(150u128, 0u128)))
}

#[allow(unused)]
pub fn test_withdraw() {
    initialize_dummy();
    let main_account_one: AccountId = get_account("first_account");
    assert_eq!(
        lock_storage_and_withdraw(main_account_one.clone(), AssetId::POLKADEX, 50u128),
        Ok(())
    );
    let balance = lock_storage_and_get_balances(main_account_one.clone(), AssetId::POLKADEX);
    assert_eq!(balance, Ok(Balances::from(50u128, 0u128)));

    //Test Error
    assert_eq!(
        lock_storage_and_withdraw(main_account_one, AssetId::POLKADEX, 200u128),
        Err(GatewayError::NotEnoughFreeBalance)
    );
}

//Test PolkadexBalanceStorage implemented Methods
#[allow(unused)]
pub fn test_set_free_balance() {
    let new_account_one: AccountId = get_account("first_account");
    let key_new = PolkadexBalanceKey::from(AssetId::POLKADEX, new_account_one.clone());
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
    let new_account_one: AccountId = get_account("new_account");
    let key_new = PolkadexBalanceKey::from(AssetId::POLKADEX, new_account_one.clone());
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
