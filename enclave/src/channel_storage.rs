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

use lazy_static::lazy_static;
use polkadex_sgx_primitives::types::SignedOrder;
use polkadex_sgx_primitives::AccountId;
use polkadex_sgx_primitives::BalancesData;
use sp_std::prelude::*;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, SgxMutex, SgxMutexGuard,
};

lazy_static! {
    static ref GLOBAL_CHANNEL_STORAGE: Arc<SgxMutex<Option<ChannelStorage>>> =
        Arc::new(SgxMutex::new(None));
}

#[derive(Clone)]
pub struct ChannelStorage {
    pub sender: Sender<ChannelType>,
}

pub fn create_channel_get_receiver() -> Result<Receiver<ChannelType>, ChannelStorageError> {
    let (sender, receiver) = channel();
    let mut storage = GLOBAL_CHANNEL_STORAGE
        .lock()
        .map_err(|_| ChannelStorageError::CouldNotGetMutex)?;
    *storage = Some(ChannelStorage { sender });
    Ok(receiver)
}

pub enum ChannelType {
    Nonce(AccountId, u32),
    Balances(Vec<BalancesData>),
    Order(SignedOrder),
}

pub fn load_sender() -> Result<Sender<ChannelType>, ChannelStorageError> {
    // Acquire lock on channel storage
    Ok(if let Some(storage) = load_channel_storage()?.clone() {
        storage
    } else {
        return Err(ChannelStorageError::ChannelNotInitialized);
    }
    .sender)
}

pub fn load_channel_storage(
) -> Result<SgxMutexGuard<'static, Option<ChannelStorage>>, ChannelStorageError> {
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
    /// Channel Not Initialized
    ChannelNotInitialized,
}
