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



pub mod tests {
    use super::*;
    use codec::Encode;


    pub fn test_initialize_and_lock_storage() {
        // given
        CancelOrderCache::initialize();

        // when
        let mutex = CancelOrderCache::load().unwrap();

        // then
        mutex.lock().unwrap();
    }

    pub fn test_insert_order_and_increment() {
        // given
        CancelOrderCache::initialize();
        let mut cache = CancelOrderCache::load()
            .unwrap()
            .lock()
            .unwrap();
        let order_uuid: OrderUUID = "hello_world".encode();
        assert_eq!(cache.request_id(), 0);

        // when
        assert!(cache.insert_order(order_uuid.clone()));

        // then
        assert_eq!(cache.request_id(), 1);
        assert!(cache.contains(&order_uuid));
    }

    /// inserts two orders
    /// removes the second, but leaves the first
    /// then checks if second was really removed
    pub fn test_remove_order() {

        // given
        CancelOrderCache::initialize();
        let mut cache = CancelOrderCache::load()
            .unwrap()
            .lock()
            .unwrap();
        let order_uuid_0: OrderUUID = "hello_world".encode();
        let order_uuid_1: OrderUUID = "hello_world_two".encode();
        let order_0_id = cache.request_id();
        assert_eq!(order_0_id, 0);
        assert!(cache.insert_order(order_uuid_0.clone()));
        let order_1_id = cache.request_id();
        assert_eq!(order_1_id, 1);
        assert!(cache.insert_order(order_uuid_1.clone()));

        // when
        assert!(cache.remove_order(&order_uuid_1));

        // then
        assert!(!cache.contains(&order_uuid_1));
        assert!(cache.contains(&order_uuid_0));
        assert_eq!(cache.request_id(), 2);
    }

    /// inserts two orders
    /// removes the second, but leaves the first
    /// then checks if second was really removed
    /// but with different cache loads to ensure
    /// state is consisted between different threads
    pub fn test_reload_cache() {
        // given
        let order_uuid_0: OrderUUID = "hello_world".encode();
        let order_uuid_1: OrderUUID = "hello_world_two".encode();
        {
            CancelOrderCache::initialize();
        }
        {
            let mut cache = CancelOrderCache::load()
                .unwrap()
                .lock()
                .unwrap();
            let order_0_id = cache.request_id();
            assert_eq!(order_0_id, 0);
            assert!(cache.insert_order(order_uuid_0.clone()));
            let order_1_id = cache.request_id();
            assert_eq!(order_1_id, 1);
            assert!(cache.insert_order(order_uuid_1.clone()));
        }

        // when
        {
            let mut cache = CancelOrderCache::load()
                .unwrap()
                .lock()
                .unwrap();
            assert!(cache.remove_order(&order_uuid_1));
        }


        // then
        let cache = CancelOrderCache::load()
            .unwrap()
            .lock()
            .unwrap();
        assert!(!cache.contains(&order_uuid_1));
        assert!(cache.contains(&order_uuid_0));
        assert_eq!(cache.request_id(), 2);
    }

}