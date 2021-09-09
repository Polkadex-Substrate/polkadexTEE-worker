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
use lazy_static::lazy_static;
use polkadex_sgx_primitives::types::SignedOrder;
use polkadex_sgx_primitives::AccountId;
use sp_std::prelude::*;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, SgxMutex, SgxMutexGuard,
};

lazy_static! {
    static ref GLOBAL_CHANNEL_STORAGE: Arc<SgxMutex<Option<ChannelStorage>>> =
        Arc::new(SgxMutex::new(None));
}

lazy_static! {
    static ref GLOBAL_CHANNEL_STORAGE_TEST: Arc<SgxMutex<Option<ChannelStorage>>> =
        Arc::new(SgxMutex::new(None));
}

#[derive(Clone)]
pub struct ChannelStorage {
    pub sender: Sender<ChannelType>,
}

pub fn create_channel_get_receiver() -> Result<Receiver<ChannelType>, ChannelStorageError> {
    internal_create_channel_get_receiver(false)
}

pub fn mock_create_channel_get_receiver() -> Result<Receiver<ChannelType>, ChannelStorageError> {
    internal_create_channel_get_receiver(true)
}

fn internal_create_channel_get_receiver(
    mock: bool,
) -> Result<Receiver<ChannelType>, ChannelStorageError> {
    let (sender, receiver) = channel();

    if mock {
        let mut storage = GLOBAL_CHANNEL_STORAGE_TEST
            .lock()
            .map_err(|_| ChannelStorageError::CouldNotGetMutex)?;
        *storage = Some(ChannelStorage { sender });
    } else {
        let mut storage = GLOBAL_CHANNEL_STORAGE
            .lock()
            .map_err(|_| ChannelStorageError::CouldNotGetMutex)?;
        *storage = Some(ChannelStorage { sender });
    }

    Ok(receiver)
}

pub enum ChannelType {
    Nonce(AccountId, u32),
    Balances(PolkadexBalanceKey, Balances),
    Order(SignedOrder),
}

pub fn load_sender() -> Result<Sender<ChannelType>, ChannelStorageError> {
    internal_load_sender(false)
}

pub fn mock_load_sender() -> Result<Sender<ChannelType>, ChannelStorageError> {
    internal_load_sender(true)
}

fn internal_load_sender(mock: bool) -> Result<Sender<ChannelType>, ChannelStorageError> {
    // Acquire lock on channel storage
    Ok(if let Some(storage) = {
        if mock {
            mock_load_channel_storage()
        } else {
            load_channel_storage()
        }
    }?
    .clone()
    {
        storage
    } else {
        return Err(ChannelStorageError::ChannelNotInitialized);
    }
    .sender)
}

pub fn load_channel_storage(
) -> Result<SgxMutexGuard<'static, Option<ChannelStorage>>, ChannelStorageError> {
    internal_load_channel_storage(false)
}

pub fn mock_load_channel_storage(
) -> Result<SgxMutexGuard<'static, Option<ChannelStorage>>, ChannelStorageError> {
    internal_load_channel_storage(true)
}

fn internal_load_channel_storage(
    mock: bool,
) -> Result<SgxMutexGuard<'static, Option<ChannelStorage>>, ChannelStorageError> {
    if mock {
        GLOBAL_CHANNEL_STORAGE_TEST
            .lock()
            .map_err(|_| ChannelStorageError::CouldNotGetMutex)
    } else {
        GLOBAL_CHANNEL_STORAGE
            .lock()
            .map_err(|_| ChannelStorageError::CouldNotGetMutex)
    }
}

#[derive(Eq, Debug, PartialEq, PartialOrd)]
pub enum ChannelStorageError {
    /// Could not load the storage for some reason
    CouldNotLoadStorage,
    /// Could not get mutex
    CouldNotGetMutex,
    /// Channel Not Initialized
    ChannelNotInitialized,
}

pub mod tests {
    use super::{mock_create_channel_get_receiver, mock_load_channel_storage, mock_load_sender};

    pub fn test_create_channel_get_receiver() {
        assert!(mock_create_channel_get_receiver().is_ok())
    }

    pub fn test_load_channel_storage() {
        mock_create_channel_get_receiver().unwrap();
        assert!(mock_load_channel_storage().unwrap().is_some())
    }

    pub fn test_load_sender() {
        mock_create_channel_get_receiver().unwrap();
        assert!(mock_load_sender().is_ok())
    }
}
