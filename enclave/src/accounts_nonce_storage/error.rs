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

use derive_more::{Display, From};
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Display, From, PartialEq, Eq)]
pub enum Error {
    /// Could not load the registry for some reason
    CouldNotLoadRegistry,
    /// Could not get mutex
    CouldNotGetMutex,
    /// Main account is already registered
    AccountAlreadyRegistered,
    /// Main account is not registered
    AccountNotRegistered,
    /// The proxy is already registered
    ProxyAlreadyRegistered,
    /// The proxy is not registered
    ProxyNotRegistered,
    /// Nonce is not initialized
    NonceUninitialized,
    /// Nonce validation failed (didn't match)
    NonceValidationFailed,
}
