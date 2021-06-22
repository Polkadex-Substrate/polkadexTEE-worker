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

use crate::openfinex::openfinex_types::RequestId;
use core::sync::atomic::{AtomicU64, Ordering};

/// a trait to generate a sequence of request ids
/// @obsolete
/// FIXME this trait and struct is probably obsolete, since the request ID gets
/// generated in the polkadex gateway cache, remove!
pub trait RequestIdGenerator {
    fn generate_next_id(&self) -> RequestId;
}

/// implementation to return a counting request id, starting at 1
pub struct CountingRequestIdGenerator {}

impl RequestIdGenerator for CountingRequestIdGenerator {
    fn generate_next_id(&self) -> RequestId {
        static REQUEST_ID: AtomicU64 = AtomicU64::new(1); // thread safe counter
        REQUEST_ID.fetch_add(1, Ordering::SeqCst) as u128
    }
}
