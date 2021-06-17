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

pub use crate::openfinex::openfinex_types::RequestId;
use log::*;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::sync::SgxMutex;

/// result type definition using the OpenFinexApiError$
/// -> This might be sensible to change to custom error in the future
/// But for now only two tpyes of errors are necessary..
pub type CacheResult<T> = core::result::Result<T, ()>;

/// Generic cache provider, requires to be initialized before load
pub trait CacheProvider<T> {
    fn initialize(&self);

    fn load(&self) -> CacheResult<&'static SgxMutex<T>>;
}

/// A static cache provider, meaning it uses global state to store the cache,
/// stored in an atomic pointer, guarded by an SGX mutex
pub struct StaticCacheProvider<T: 'static> {
    initial_item: &'static dyn Fn() -> T,
    static_cache_ptr: &'static AtomicPtr<()>,
}

impl<T> CacheProvider<T> for StaticCacheProvider<T> {
    fn initialize(&self) {
        let initial = (self.initial_item)();
        let cache_storage_ptr = Arc::new(SgxMutex::new(initial));
        let cache_ptr = Arc::into_raw(cache_storage_ptr);
        self.static_cache_ptr
            .store(cache_ptr as *mut (), Ordering::SeqCst);
    }

    fn load(&self) -> CacheResult<&'static SgxMutex<T>> {
        let ptr = self.static_cache_ptr.load(Ordering::SeqCst) as *mut SgxMutex<T>;
        if ptr.is_null() {
            error!("Could not load cache");
            return Err(());
        } else {
            Ok(unsafe { &*ptr })
        }
    }
}

impl<T> StaticCacheProvider<T> {
    /// constructor taking a generator function for the initial item
    /// and a generic atomic pointer to where the cache should be stored
    pub fn new(
        initial_item: &'static dyn Fn() -> T,
        static_cache_ptr: &'static AtomicPtr<()>,
    ) -> Self {
        StaticCacheProvider {
            initial_item,
            static_cache_ptr,
        }
    }
}

/// A cache provider that stores the cache in local state, not a global static pointer
pub struct LocalCacheProvider<T: 'static> {
    initial_cache: &'static dyn Fn() -> T,
    cache_ptr: AtomicPtr<()>,
}

impl<T> LocalCacheProvider<T> {
    /// constructor taking a generator function for the initial item
    /// and a generic atomic pointer to where the cache should be stored
    pub fn new(initial_cache: &'static dyn Fn() -> T) -> Self {
        let cache_provider = LocalCacheProvider {
            initial_cache,
            cache_ptr: AtomicPtr::new(0 as *mut ()),
        };

        cache_provider.initialize();

        cache_provider
    }
}

impl<T> CacheProvider<T> for LocalCacheProvider<T> {
    fn initialize(&self) {
        let initial_cache = (self.initial_cache)();
        let cache_storage_ptr = Arc::new(SgxMutex::new(initial_cache));
        let cache_ptr = Arc::into_raw(cache_storage_ptr);
        self.cache_ptr.store(cache_ptr as *mut (), Ordering::SeqCst);
    }

    fn load(&self) -> CacheResult<&'static SgxMutex<T>> {
        let ptr = self.cache_ptr.load(Ordering::SeqCst) as *mut SgxMutex<T>;
        if ptr.is_null() {
            error!("Could not load cache");
            return Err(());
        } else {
            Ok(unsafe { &*ptr })
        }
    }
}

/// Static Storage Interaction trait - used to initialize and load the storage to be
/// used from different threads.
/// @deprecated -> move all implementations to CacheProvider<T> or StaticCacheProvider<T> respectively
pub trait StaticStorageApi {
    /// initializes the storage within a static pointer to be usable from different threads
    fn initialize();
    /// initializes the storage within a static pointer to be usable from different threads
    fn load() -> CacheResult<&'static SgxMutex<Self>>;
}
