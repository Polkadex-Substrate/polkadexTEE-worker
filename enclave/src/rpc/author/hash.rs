// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Extrinsic helpers for author RPC module.

extern crate alloc;
use alloc::vec::Vec;
use codec::{Decode, Encode};

use substratee_stf::TrustedOperation;

/// RPC Trusted call or hash
///
/// Allows to refer to trusted calls either by its raw representation or its hash.
#[derive(Debug, Encode, Decode)]
pub enum TrustedOperationOrHash<Hash> {
    /// The hash of the call.
    Hash(Hash),
    /// Raw extrinsic bytes.
    OperationEncoded(Vec<u8>),
    /// Raw extrinsic
    Operation(TrustedOperation),
}
