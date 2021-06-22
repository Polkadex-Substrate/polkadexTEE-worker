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

use codec::Encode;
use polkadex_sgx_primitives::types::{DirectRequest, MarketId, Order, CancelOrder, OrderSide, OrderType, OrderUUID};
use polkadex_sgx_primitives::{AccountId, AssetId};
use sp_core::{ed25519 as ed25519_core, Pair, H256};
use substratee_stf::{KeyPair, TrustedCall, TrustedCallSigned};

pub fn create_dummy_request() -> DirectRequest {
    DirectRequest {
        encoded_text: vec![0, 1, 2, 3, 4],
        shard: H256::from([1u8; 32]),
    }
}

pub fn create_dummy_account() -> ed25519_core::Pair {
    ed25519_core::Pair::from_seed(b"12345678901234567890123456789012")
}

pub fn create_dummy_order(account: AccountId) -> Order {
    Order {
        user_uid: account,
        market_id: MarketId {
            quote: AssetId::DOT,
            base: AssetId::POLKADEX,
        },
        side: OrderSide::ASK,
        market_type: "trusted".encode(),
        order_type: OrderType::MARKET,
        quantity: 5000,
        price: None,
    }
}

pub fn create_dummy_cancel_order(account: AccountId, order_id: OrderUUID) -> CancelOrder {
    CancelOrder {
        user_uid: account,
        market_id: MarketId {
            quote: AssetId::DOT,
            base: AssetId::POLKADEX,
        },
        order_id
    }
}

// sign a trusted call - use only for test/dummy cases!
pub fn sign_trusted_call(
    trusted_call: TrustedCall,
    signer: ed25519_core::Pair,
) -> TrustedCallSigned {
    let mr_enclave = [2u8; 32];
    let shard_identifier = H256::from(mr_enclave);

    trusted_call.sign(&KeyPair::Ed25519(signer), 0, &mr_enclave, &shard_identifier)
}
