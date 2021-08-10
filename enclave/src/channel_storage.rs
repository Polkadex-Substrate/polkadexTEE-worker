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
    static ref GLOBAL_CHANNEL_STORAGE: Arc<SgxMutex<ChannelStorage>> =
        Arc::new(SgxMutex::new(ChannelStorage::default()));
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
    Order(SignedOrder),
}

pub fn load_sender() -> Result<Sender<ChannelType>, ChannelStorageError> {
    // Acquire lock on channel storage
    let storage = load_channel_storage()?;

    let result = storage.sender.clone();
    Ok(result)
}

pub fn load_receiver() -> Result<Arc<SgxMutex<Receiver<ChannelType>>>, ChannelStorageError> {
    // Acquire lock on channel storage
    let storage = load_channel_storage()?;

    let result = storage.receiver.clone();
    Ok(result)
}

pub fn load_channel_storage() -> Result<SgxMutexGuard<'static, ChannelStorage>, ChannelStorageError>
{
    GLOBAL_CHANNEL_STORAGE
        .lock()
        .map_err(|_| ChannelStorageError::CouldNotGetMutex)
}

#[derive(Eq, Debug, PartialEq, PartialOrd)]
pub enum ChannelStorageError {
    /// Could not load the storage for some reason
    CouldNotLoadStorage,
    /// Could not get mutex
    CouldNotGetMutex,
}
