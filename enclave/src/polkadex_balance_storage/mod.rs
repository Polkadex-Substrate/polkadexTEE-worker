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

use crate::polkadex_gateway::GatewayError;
use log::*;
use polkadex_sgx_primitives::{AccountId, AssetId, Balance};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

static GLOBAL_POLKADEX_BALANCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub mod balance_storage;
pub mod balances;
pub mod polkadex_balance_key;

pub use balance_storage::*;
pub use balances::*;
pub use polkadex_balance_key::*;

pub fn create_in_memory_balance_storage() -> Result<(), GatewayError> {
    let balances_storage = PolkadexBalanceStorage::create();
    let storage_ptr = Arc::new(SgxMutex::<PolkadexBalanceStorage>::new(balances_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_POLKADEX_BALANCE_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub fn load_balance_storage() -> Result<&'static SgxMutex<PolkadexBalanceStorage>, GatewayError> {
    let ptr = GLOBAL_POLKADEX_BALANCE_STORAGE.load(Ordering::SeqCst)
        as *mut SgxMutex<PolkadexBalanceStorage>;
    if ptr.is_null() {
        error!("Pointer is Null");
        return Err(GatewayError::UnableToLoadPointer);
    }
    Ok(unsafe { &*ptr })
}

pub fn lock_storage_and_reserve_balance(
    main_acc: &AccountId,
    token: AssetId,
    amount: Balance,
) -> Result<(), GatewayError> {
    // Acquire lock on balance_storagepolkadex_balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> =
        mutex.lock().map_err(|_| {
            error!("Could not lock mutex of balance storage");
            GatewayError::UnableToLock
        })?;
    let balance = match balance_storage.read_balance(token.clone(), main_acc.clone()) {
        Some(balance) => balance.clone(),
        None => {
            error!("Account does not have a balance storage for this asset id yet");
            return Err(GatewayError::NotEnoughFreeBalance);
        }
    };
    if balance.free < amount {
        error!("Not enough free balance: Expected {:?}, available: {:?} of token {:?}", amount, balance.free, token);
        return Err(GatewayError::NotEnoughFreeBalance);
    }
    balance_storage.set_free_balance(
        token.clone(),
        main_acc.clone(),
        balance.free.saturating_sub(amount),
    )?;
    balance_storage.set_reserve_balance(
        token.clone(),
        main_acc.clone(),
        balance.reserved.saturating_add(amount),
    )?;
    Ok(())
}

pub fn lock_storage_unreserve_balance(
    main_acc: &AccountId,
    token: AssetId,
    amount: Balance,
) -> Result<(), GatewayError> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> =
        mutex.lock().map_err(|_| {
            error!("Could not lock mutex of balance storage");
            GatewayError::UnableToLock
        })?;
    let balance = match balance_storage.read_balance(token.clone(), main_acc.clone()) {
        Some(balance) => balance.clone(),
        None => {
            error!("Account does not have a balance storage for this asset id yet");
            return Err(GatewayError::NotEnoughFreeBalance);
        }
    };
    if balance.reserved < amount {
        error!("Unable to un-reserve balance greater than reserved balance");
        return Err(GatewayError::NotEnoughReservedBalance);
    }
    balance_storage.set_free_balance(
        token.clone(),
        main_acc.clone(),
        balance.free.saturating_add(amount),
    )?;
    balance_storage.set_reserve_balance(
        token,
        main_acc.clone(),
        balance.reserved.saturating_sub(amount),
    )?;
    Ok(())
}

pub fn lock_storage_and_deposit(
    main_acc: AccountId,
    token: AssetId,
    amt: Balance,
) -> Result<(), GatewayError> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> =
        mutex.lock().map_err(|_| {
            error!("Could not lock mutex of balance storage");
            GatewayError::UnableToLock
        })?;
    balance_storage.deposit(token, main_acc, amt)
}

pub fn lock_storage_and_withdraw(
    main_acc: AccountId,
    token: AssetId,
    amt: Balance,
) -> Result<(), GatewayError> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> =
        mutex.lock().map_err(|_| {
            error!("Could not lock mutex of balance storage");
            GatewayError::UnableToLock
        })?;
    match balance_storage.read_balance(token.clone(), main_acc.clone()) {
        Some(balance) => {
            if balance.free >= amt {
                balance_storage.withdraw(token, main_acc, amt)?;
            } else {
                error!("Balance is low");
                return Err(GatewayError::NotEnoughFreeBalance);
            }
        }
        None => {
            error!("Account Id or Asset Id is not available");
            return Err(GatewayError::AccountIdOrAssetIdNotFound);
        }
    }
    Ok(())
}

pub fn lock_storage_and_initialize_balance(
    main_acc: AccountId,
    token: AssetId,
) -> Result<(), GatewayError> {
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> =
        mutex.lock().map_err(|_| {
            error!("Could not lock mutex of balance storage");
            GatewayError::UnableToLock
        })?;
    balance_storage.initialize_balance(token, main_acc, 0);
    Ok(())
}

pub fn lock_storage_and_get_balances(
    main_acc: AccountId,
    token: AssetId,
) -> Result<Balances, GatewayError> {
    let mutex = load_balance_storage()?;
    let balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = mutex.lock().map_err(|_| {
        error!("Could not lock mutex of balance storage");
        GatewayError::UnableToLock
    })?;
    if let Some(balance) = balance_storage.read_balance(token, main_acc).cloned() {
        Ok(balance)
    } else {
        error!("Account Id or Asset Id is not available");
        Err(GatewayError::AccountIdOrAssetIdNotFound)
    }
}

pub fn lock_storage_transfer_balance(
    from: &AccountId,
    to: &AccountId,
    token: AssetId,
    amount: u128,
) -> Result<(), GatewayError> {
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> =
        mutex.lock().map_err(|_| {
            error!("Could not lock mutex of balance storage");
            GatewayError::UnableToLock
        })?;
    balance_storage.reduce_free_balance(token.clone(), from.clone(), amount)?;
    balance_storage.increase_free_balance(token.clone(), to.clone(), amount)?;
    Ok(())
}
// Ony for testing
pub fn lock_storage_increase_free_balance(
    token: AssetId,
    account: AccountId,
    amount: u128,
) -> Result<(), GatewayError> {
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> =
        mutex.lock().map_err(|_| {
            error!("Could not lock mutex of balance storage");
            GatewayError::UnableToLock
        })?;
    balance_storage.increase_free_balance(token.clone(), account.clone(), amount)?;
    Ok(())
}
