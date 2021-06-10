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
use polkadex_sgx_primitives::types::Order;
use std::collections::HashMap;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, SgxMutex};
use crate::polkadex_cache::cache_api::{RequestId, StaticStorageApi, CacheResult};

use codec::{Encode, Decode};
use std::string::{String, ToString};

static CREATE_ORDER_CACHE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());


#[derive(Debug)]
pub struct CreateOrderCache {
    /// The set of chached order uuids
    order_map: HashMap<RequestId, Order>,
    /// Nonce / request_id
    request_id: RequestId,
}

impl Default for CreateOrderCache{
    fn default() -> Self {
        CreateOrderCache {
            order_map: Default::default(),
            request_id: 0,
        }
    }
}

impl StaticStorageApi for CreateOrderCache {
    fn initialize() {
        let cache = CreateOrderCache {
            order_map: HashMap::new(),
            request_id: 0,
        };
        let cache_storage_ptr = Arc::new(SgxMutex::new(cache));
        let cache_ptr = Arc::into_raw(cache_storage_ptr);
        CREATE_ORDER_CACHE.store(cache_ptr as *mut (), Ordering::SeqCst);
    }

    fn load() -> CacheResult<&'static SgxMutex<Self>> {
        let ptr = CREATE_ORDER_CACHE.load(Ordering::SeqCst) as *mut SgxMutex<Self>;
        if ptr.is_null() {
            error!("Could not load create order cache");
            return Err(());
        } else {
            Ok(unsafe { &*ptr })
        }
    }
}


impl CreateOrderCache {
    /// removes the given order from the cache. Returns the value of
    /// the given key if previously present
    pub fn remove_order(&mut self, id: &RequestId) -> Option<Order> {
        self.order_map.remove(id)
    }

    /// inserts an order to the set and increments the request id.
    /// Returns old value if it was previously present
    pub fn insert_order(&mut self, order: Order) -> Option<Order> {
        let result = self.order_map.insert(self.request_id, order);
        self.increment_request_id();
        result
    }

    pub fn request_id(&self) -> RequestId {
        self.request_id
    }
}

impl CreateOrderCache {
    fn increment_request_id(&mut self) {
        self.request_id = self.request_id.saturating_add(1)
    }
}


pub mod tests {
    use super::*;
    use crate::test_orderbook_storage;


    pub fn test_initialize_and_lock_storage() {
        // given
        CreateOrderCache::initialize();

        // when
        let mutex = CreateOrderCache::load().unwrap();

        // then
        mutex.lock().unwrap();
    }

    pub fn test_insert_order_and_increment() {
        // given
        CreateOrderCache::initialize();
        let mut cache = CreateOrderCache::load()
            .unwrap()
            .lock()
            .unwrap();
        let orders = test_orderbook_storage::get_dummy_orders();
        assert_eq!(cache.request_id(), 0);

        // when
        let none = cache.insert_order(orders[0].clone());

        // then
        assert!(none.is_none()); // no previously inserted order should be there
        assert_eq!(cache.request_id(), 1);
    }

    pub fn test_remove_order() {
        // given
        CreateOrderCache::initialize();
        let mut cache = CreateOrderCache::load()
            .unwrap()
            .lock()
            .unwrap();
        let orders = test_orderbook_storage::get_dummy_orders();
        let order_0_id = cache.request_id();
        assert_eq!(order_0_id, 0);
        cache.insert_order(orders[0].clone());
        let order_1_id = cache.request_id();
        assert_eq!(order_1_id, 1);
        cache.insert_order(orders[1].clone());

        // when
        let order_1 = cache.remove_order(&order_1_id).unwrap();
        let none = cache.remove_order(&order_1_id);

        // then
        assert!(none.is_none());
        assert_eq!(order_1, orders[1]);
        assert_eq!(cache.request_id(), 2);
    }

    pub fn test_reload_cache() {
        // given
        let orders = test_orderbook_storage::get_dummy_orders();
        {
            CreateOrderCache::initialize();
        }
        {
            let mut cache = CreateOrderCache::load()
                .unwrap()
                .lock()
                .unwrap();
            let order_0_id = cache.request_id();
            assert_eq!(order_0_id, 0);
            cache.insert_order(orders[0].clone());
            let order_1_id = cache.request_id();
            assert_eq!(order_1_id, 1);
            cache.insert_order(orders[1].clone());
        }

        // when
        {
            let mut cache = CreateOrderCache::load()
                .unwrap()
                .lock()
                .unwrap();
            let order_1 = cache.remove_order(&1).unwrap();
            let none = cache.remove_order(&1);

            assert!(none.is_none());
            assert_eq!(order_1, orders[1]);
        }


        // then
        let mut cache = CreateOrderCache::load()
            .unwrap()
            .lock()
            .unwrap();
        let order_1 = cache.remove_order(&1);
        assert!(order_1.is_none());
        let order_0 = cache.remove_order(&0).unwrap();
        assert_eq!(order_0, orders[0]);
        assert_eq!(cache.request_id(), 2);
    }

}