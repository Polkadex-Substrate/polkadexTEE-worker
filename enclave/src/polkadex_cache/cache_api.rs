// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü.
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


use std::sync::SgxMutex;
pub use crate::openfinex::openfinex_types::RequestId;


/// result type definition using the OpenFinexApiError$
/// -> This might be sensible to change to custom error in the future
/// But for now only two tpyes of errors are necessary..
pub type CacheResult<T> = core::result::Result<T, ()>;


/// Static Storage Interaction trait - used to initialize and load the storage to be
/// used from different threads.
pub trait StaticStorageApi {
    /// initializes the storage within a static pointer to be usable from different threads
    fn initialize();
    /// initializes the storage within a static pointer to be usable from different threads
    fn load() -> CacheResult<&'static SgxMutex<Self>>;
}
