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

pub const RSA3072_SEALED_KEY_FILE: &str = "rsa3072_key_sealed.bin";
pub const SEALED_SIGNER_SEED_FILE: &str = "ed25519_key_sealed.bin";
pub const ENCRYPTED_STATE_FILE: &str = "state.bin";
pub const SHARDS_PATH: &str = "./shards";
pub const AES_KEY_FILE_AND_INIT_V: &str = "aes_key_sealed.bin";
pub const CHAIN_RELAY_DB: &str = "chain_relay_db.bin";

/// Polkadex Defines 1 token as 10^^18 fundamental units
pub const UNIT: u128 = 1000000000000000000;
pub const RA_DUMP_CERT_DER_FILE: &str = "ra_dump_cert.der";

#[cfg(feature = "production")]
pub static RA_SPID_FILE: &str = "../bin/spid_production.txt";
#[cfg(feature = "production")]
pub static RA_API_KEY_FILE: &str = "../bin/key_production.txt";

#[cfg(not(feature = "production"))]
pub static RA_SPID_FILE: &str = "../bin/spid.txt";
#[cfg(not(feature = "production"))]
pub static RA_API_KEY_FILE: &str = "../bin/key.txt";

// you may have to update these indices upon new builds of the runtime
// you can get the index from metadata, counting modules starting with zero
pub static SUBSRATEE_REGISTRY_MODULE: u8 = 38u8;
pub static REGISTER_ENCLAVE: u8 = 0u8;
//pub static UNREGISTER_ENCLAVE: u8 = 1u8;
pub static CALL_WORKER: u8 = 2u8;
pub static CALL_CONFIRMED: u8 = 3u8;
pub static BLOCK_CONFIRMED: u8 = 4u8;
pub static SHIELD_FUNDS: u8 = 5u8;

// Polkadex Module Constants ( this should be updated if order of modules in the runtime is changed
pub static OCEX_MODULE: u8 = 39u8;
pub static OCEX_REGISTER: u8 = 3u8;
pub static OCEX_ADD_PROXY: u8 = 4u8;
pub static OCEX_REMOVE_PROXY: u8 = 5u8;
pub static OCEX_DEPOSIT: u8 = 0u8;
pub static OCEX_RELEASE: u8 = 1u8;
pub static OCEX_WITHDRAW: u8 = 2u8;

// bump this to be consistent with SubstraTEE-node runtime
pub static RUNTIME_SPEC_VERSION: u32 = 265;
pub static RUNTIME_TRANSACTION_VERSION: u32 = 2;

// timeouts for getter and call execution
pub static CALLTIMEOUT: i64 = 300; // timeout in ms
pub static GETTERTIMEOUT: i64 = 300; // timeout in ms
