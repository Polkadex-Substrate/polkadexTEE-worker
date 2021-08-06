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

use crate::polkadex_balance_storage::{Balances, PolkadexBalanceKey};
use crate::polkadex_gateway::GatewayError;
use log::*;
use polkadex_sgx_primitives::AccountId;
use sp_std::prelude::*;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc, SgxMutex, SgxMutexGuard,
};

static GLOBAL_CHANNEL_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn create_in_memory_channel_storage() -> Result<(), GatewayError> {
    let storage = ChannelStorage::default();
    let storage_ptr = Arc::new(SgxMutex::<ChannelStorage>::new(storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_CHANNEL_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}

pub struct ChannelStorage {
    pub sender: Sender<ChannelType>,
    pub receiver: Arc<SgxMutex<Receiver<ChannelType>>>,
}

impl Default for ChannelStorage {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Arc::new(SgxMutex::new(receiver)),
        }
    }
}

pub enum ChannelType {
    Nonce(AccountId, u32),
    Balances(PolkadexBalanceKey, Balances),
}

pub fn load_sender() -> Result<Sender<ChannelType>, ChannelStorageError> {
    // Acquire lock on proxy_registry
    let mutex = load_channel_storage()?;
    let storage: SgxMutexGuard<ChannelStorage> = mutex
        .lock()
        .map_err(|_| ChannelStorageError::CouldNotGetMutex)?;
    let result = storage.sender.clone();
    Ok(result)
}

pub fn load_receiver() -> Result<Arc<SgxMutex<Receiver<ChannelType>>>, ChannelStorageError> {
    // Acquire lock on proxy_registry
    let mutex = load_channel_storage()?;
    let storage: SgxMutexGuard<ChannelStorage> = mutex
        .lock()
        .map_err(|_| ChannelStorageError::CouldNotGetMutex)?;
    let result = storage.receiver.clone();
    Ok(result)
}

pub fn load_channel_storage() -> Result<&'static SgxMutex<ChannelStorage>, ChannelStorageError> {
    let ptr = GLOBAL_CHANNEL_STORAGE.load(Ordering::SeqCst) as *mut SgxMutex<ChannelStorage>;
    if ptr.is_null() {
        error!("Null pointer to polkadex account registry");
        Err(ChannelStorageError::CouldNotLoadStorage)
    } else {
        Ok(unsafe { &*ptr })
    }
}

#[derive(Eq, Debug, PartialEq, PartialOrd)]
pub enum ChannelStorageError {
    /// Could not load the storage for some reason
    CouldNotLoadStorage,
    /// Could not get mutex
    CouldNotGetMutex,
}
