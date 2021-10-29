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

use codec::Encode;
use polkadex_sgx_primitives::{accounts::get_account, AccountId, AssetId};
use sgx_tstd::sync::SgxMutexGuard;
use sp_std::prelude::*;

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

//Test PolkadexBalanceStorage lock Methods
#[allow(unused)]
pub fn test_lock_storage_and_reserve_balance() {
    initialize_dummy();
    let main_account: AccountId = get_account("first_account");
    let mut polkadex_balance_storage = PolkadexBalanceStorage::create();
    polkadex_balance_storage.set_free_balance(AssetId::POLKADEX, main_account.clone(), 100u128);
    lock_storage_and_reserve_balance(&main_account, AssetId::POLKADEX, 50u128);
    assert_eq!(
        lock_storage_and_get_balances(main_account, AssetId::POLKADEX),
        Ok(Balances::from(50u128, 50u128))
    )
}

#[allow(unused)]
pub fn test_lock_storage_unreserve_balance() {
    initialize_dummy();
    let main_account: AccountId = get_account("first_account");
    assert_eq!(
        lock_storage_and_reserve_balance(&main_account, AssetId::POLKADEX, 100u128),
        Ok(())
    );
    lock_storage_unreserve_balance(&main_account, AssetId::POLKADEX, 50u128);
    assert_eq!(
        lock_storage_and_get_balances(main_account, AssetId::POLKADEX),
        Ok(Balances::from(50u128, 50u128))
    )
}

#[allow(unused)]
pub fn test_lock_storage_and_initialize_balance() {
    initialize_dummy();
    let main_account: AccountId = get_account("first_account");
    assert_eq!(
        lock_storage_and_initialize_balance(main_account.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        lock_storage_and_get_balances(main_account, AssetId::POLKADEX),
        Ok(Balances::from(0u128, 0u128))
    )
}

#[allow(unused)]
pub fn test_lock_storage_and_deposit() {
    initialize_dummy();
    let main_account: AccountId = get_account("first_account");
    assert_eq!(
        lock_storage_and_initialize_balance(main_account.clone(), AssetId::POLKADEX),
        Ok(())
    );
    lock_storage_and_deposit(main_account.clone(), AssetId::POLKADEX, 50u128);
    assert_eq!(
        lock_storage_and_get_balances(main_account, AssetId::POLKADEX),
        Ok(Balances::from(50u128, 0u128))
    )
}

#[allow(unused)]
pub fn test_lock_storage_and_withdraw() {
    initialize_dummy();
    let main_account: AccountId = get_account("first_account");
    assert_eq!(
        lock_storage_and_initialize_balance(main_account.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        lock_storage_and_deposit(main_account.clone(), AssetId::POLKADEX, 100u128),
        Ok(())
    );
    lock_storage_and_withdraw(main_account.clone(), AssetId::POLKADEX, 50u128);
    assert_eq!(
        lock_storage_and_get_balances(main_account, AssetId::POLKADEX),
        Ok(Balances::from(50u128, 0u128))
    )
}

#[allow(unused)]
pub fn test_lock_storage_transfer_balance() {
    initialize_dummy();
    let main_account: AccountId = get_account("first_account");
    let secondary_account: AccountId = get_account("second_account");
    assert_eq!(
        lock_storage_and_initialize_balance(main_account.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        lock_storage_and_initialize_balance(secondary_account.clone(), AssetId::POLKADEX),
        Ok(())
    );
    assert_eq!(
        lock_storage_and_deposit(main_account.clone(), AssetId::POLKADEX, 100u128),
        Ok(())
    );
    lock_storage_transfer_balance(&main_account, &secondary_account, AssetId::POLKADEX, 50u128);
    assert_eq!(
        (
            lock_storage_and_get_balances(main_account, AssetId::POLKADEX),
            lock_storage_and_get_balances(secondary_account, AssetId::POLKADEX)
        ),
        (
            Ok(Balances::from(50u128, 0u128)),
            Ok(Balances::from(50u128, 0u128))
        )
    )
}

#[allow(unused)]
pub fn test_increase_free_balance() {
    initialize_dummy();
    let main_account: AccountId = get_account("first_account");
    assert_eq!(
        lock_storage_increase_free_balance(
            AssetId::Asset(4294967297),
            main_account.clone(),
            200u128
        ),
        Ok(())
    );
    assert_eq!(
        lock_storage_and_get_balances(main_account, AssetId::Asset(4294967297)),
        Ok(Balances::from(200u128, 0u128))
    )
}
