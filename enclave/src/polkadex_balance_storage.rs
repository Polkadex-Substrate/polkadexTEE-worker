use crate::polkadex_gateway::GatewayError;
use codec::{Decode, Encode};
use log::*;
use polkadex_sgx_primitives::{AccountId, AssetId, Balance};
use sgx_tstd::collections::HashMap;
use sgx_tstd::vec::Vec;
use sgx_types::{sgx_status_t, SgxResult};
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, SgxMutex, SgxMutexGuard,
};

static GLOBAL_POLKADEX_BALANCE_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub type EncodedKey = Vec<u8>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Balances {
    pub free: Balance,
    pub reserved: Balance,
}

impl Balances {
    pub fn from(free: Balance, reserved: Balance) -> Self {
        Self { free, reserved }
    }
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PolkadexBalanceKey {
    pub asset_id: AssetId,
    pub account_id: AccountId,
}

impl PolkadexBalanceKey {
    pub fn from(asset_id: AssetId, account_id: AccountId) -> Self {
        Self {
            asset_id,
            account_id,
        }
    }
}

pub struct PolkadexBalanceStorage {
    /// map (tokenID, AccountID) -> (balance free, balance reserved)
    pub storage: HashMap<EncodedKey, Balances>,
}

impl PolkadexBalanceStorage {
    pub fn create() -> PolkadexBalanceStorage {
        PolkadexBalanceStorage {
            storage: HashMap::new(),
        }
    }

    pub fn read_balance(&self, token: AssetId, acc: AccountId) -> Option<&Balances> {
        let key = PolkadexBalanceKey::from(token, acc).encode();
        debug!("reading balance from key: {:?}", key);
        self.storage.get(&key)
    }

    pub fn initialize_balance(&mut self, token: AssetId, acc: AccountId, free: Balance) {
        let key = PolkadexBalanceKey::from(token, acc).encode();
        debug!("creating new entry for key: {:?}", key);
        self.storage.insert(key, Balances::from(free, 0u128));
    }

    pub fn set_free_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = amt;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn set_reserve_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.reserved = amt;
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn deposit(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc.clone()).encode())
        {
            Some(balance) => {
                balance.free = balance.free.saturating_add(amt);
                Ok(())
            }
            None => {
                debug!("No entry available for given token- and AccountId, creating new.");
                self.initialize_balance(token, acc, amt);
                Ok(())
            }
        }
    }

    pub fn withdraw(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = balance.free.saturating_sub(amt);
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn reduce_free_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = balance
                    .free
                    .checked_sub(amt)
                    .ok_or(GatewayError::LimitOrderPriceNotFound)?; //FIXME Error type
                Ok(())
            }
            None => {
                error!("Account Id or Asset id not avalaible");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }

    pub fn increase_free_balance(
        &mut self,
        token: AssetId,
        acc: AccountId,
        amt: Balance,
    ) -> Result<(), GatewayError> {
        match self
            .storage
            .get_mut(&PolkadexBalanceKey::from(token, acc).encode())
        {
            Some(balance) => {
                balance.free = balance
                    .free
                    .checked_add(amt)
                    .ok_or(GatewayError::LimitOrderPriceNotFound)?; //FIXME Error Type
                Ok(())
            }
            None => {
                error!("Account Id or Asset Id not available [here]");
                return Err(GatewayError::AccountIdOrAssetIdNotFound);
            }
        }
    }
    // We can write functions which settle balances for two trades but we need to know the trade structure for it
}

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
    } else {
        Ok(unsafe { &*ptr })
    }
}

// TODO: Write test cases for this function
pub fn lock_storage_and_reserve_balance(
    main_acc: &AccountId,
    token: AssetId,
    amount: Balance,
) -> Result<(), GatewayError> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = match mutex.lock() {
        Ok(storage) => storage,
        Err(_) => {
            error!("Could not lock mutex of balance storage");
            return Err( GatewayError::UnableToLock)
        },
    };
    let balance = match balance_storage.read_balance(token.clone(), main_acc.clone()) {
        Some(balance) => balance.clone(),
        None => {
            error!("Account does not have a balance storage for this asset id yet");
            return Err(GatewayError::NotEnoughFreeBalance)
        }
    };
    if balance.free < amount {
        error!("Not enough free balance");
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

// TODO: Write test cases for this function
pub fn lock_storage_unreserve_balance(
    main_acc: &AccountId,
    token: AssetId,
    amount: Balance,
) -> Result<(), GatewayError> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = match mutex.lock() {
        Ok(storage) => storage,
        Err(_) => {
            error!("Could not lock mutex of balance storage");
            return Err( GatewayError::UnableToLock)
        },
    };
    let balance = match balance_storage.read_balance(token.clone(), main_acc.clone()) {
        Some(balance) => balance.clone(),
        None => {
            error!("Account does not have a balance storage for this asset id yet");
            return Err(GatewayError::NotEnoughFreeBalance)
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
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = match mutex.lock() {
        Ok(storage) => storage,
        Err(_) => {
            error!("Could not lock mutex of balance storage");
            return Err( GatewayError::UnableToLock)
        },
    };
    balance_storage.deposit(token, main_acc, amt)
}

pub fn lock_storage_and_withdraw(
    main_acc: AccountId,
    token: AssetId,
    amt: Balance,
) -> Result<(), GatewayError> {
    // Acquire lock on balance_storage
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = match mutex.lock() {
        Ok(storage) => storage,
        Err(_) => {
            error!("Could not lock mutex of balance storage");
            return Err( GatewayError::UnableToLock)
        },
    };
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

// TODO: Write Unit test for this function
pub fn lock_storage_and_initialize_balance(
    main_acc: AccountId,
    token: AssetId,
) -> Result<(), GatewayError> {
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = match mutex.lock() {
        Ok(storage) => storage,
        Err(_) => {
            error!("Could not lock mutex of balance storage");
            return Err( GatewayError::UnableToLock)
        },
    };
    balance_storage.initialize_balance(token, main_acc, 0);
    Ok(())
}

pub fn lock_storage_and_get_balances(
    main_acc: AccountId,
    token: AssetId,
) -> Result<Balances, GatewayError> {
    let mutex = load_balance_storage()?;
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = match mutex.lock() {
        Ok(storage) => storage,
        Err(_) => {
            error!("Could not lock mutex of balance storage");
            return Err( GatewayError::UnableToLock)
        },
    };
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
    let mut balance_storage: SgxMutexGuard<PolkadexBalanceStorage> = match mutex.lock() {
        Ok(storage) => storage,
        Err(_) => {
            error!("Could not lock mutex of balance storage");
            return Err( GatewayError::UnableToLock)
        },
    };
    balance_storage.reduce_free_balance(token.clone(), from.clone(), amount)?;
    balance_storage.increase_free_balance(token.clone(), to.clone(), amount)?;
    Ok(())
}
