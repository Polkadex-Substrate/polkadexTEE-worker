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

use log::*;
use polkadex_sgx_primitives::types::OrderUUID;
use std::collections::HashSet;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, SgxMutex};
use crate::polkadex_cache::cache_api::{RequestId, StaticStorageApi, CacheResult};

static CANCEL_ORDER_CACHE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

#[derive(Debug)]
pub struct CancelOrderCache {
    /// The set of chached order uuids
    order_uuids: HashSet<OrderUUID>,
    /// Nonce / request_id (do wee need this in the cancel_order?)
    request_id: RequestId,
}

impl Default for CancelOrderCache{
    fn default() -> Self {
        CancelOrderCache {
            order_uuids: Default::default(),
            request_id: 0,
        }
    }
}


impl StaticStorageApi for CancelOrderCache {
    fn initialize() {
        let cache = CancelOrderCache {
            order_uuids: HashSet::new(),
            request_id: 0,
        };
        let cache_storage_ptr = Arc::new(SgxMutex::new(cache));
        let cache_ptr = Arc::into_raw(cache_storage_ptr);
        CANCEL_ORDER_CACHE.store(cache_ptr as *mut (), Ordering::SeqCst);
    }

    fn load() -> CacheResult<&'static SgxMutex<Self>> {
        let ptr = CANCEL_ORDER_CACHE.load(Ordering::SeqCst) as *mut SgxMutex<CancelOrderCache>;
        if ptr.is_null() {
            error!("Could not load cancel order cache");
            return Err(());
        } else {
            Ok(unsafe { &*ptr })
        }
    }
}

/// public interface
impl CancelOrderCache {
    /// removes the given order from the cache. Returns true if the
    /// given value was present
    pub fn remove_order(&mut self, order_id: &OrderUUID) -> bool {
        self.order_uuids.remove(order_id)
    }

    /// inserts an order to the set and increments the request id.
    /// Returns false, if the value is already present
    pub fn insert_order(&mut self, order_id: OrderUUID) -> bool {
        let result = self.order_uuids.insert(order_id);
        self.increment_request_id();
        result
    }

    /// Returns true if the set contains a value.
    pub fn contains(&self, order_id: &OrderUUID) -> bool {
        self.order_uuids.contains(order_id)
    }

    pub fn request_id(&self) -> RequestId {
        self.request_id
    }
}

impl CancelOrderCache {
    fn increment_request_id(&mut self) {
        self.request_id = self.request_id.saturating_add(1)
    }
}
