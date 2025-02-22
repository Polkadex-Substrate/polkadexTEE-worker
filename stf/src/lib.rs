/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/

/////////////////////////////////////////////////////////////////////////////
#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(core_intrinsics)]
#![feature(derive_eq)]
#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate alloc;

#[cfg(feature = "std")]
extern crate clap;

use codec::{Compact, Decode, Encode};
#[cfg(feature = "std")]
use my_node_runtime::Balance;
#[cfg(feature = "std")]
pub use my_node_runtime::Index;
#[cfg(feature = "sgx")]
use sgx_runtime::Balance;
#[cfg(feature = "sgx")]
pub use sgx_runtime::Index;

use sp_core::crypto::AccountId32;
//use sp_core::{Encode, Decode};
use sp_core::{ed25519, sr25519, Pair, H256};
use sp_runtime::{traits::Verify, MultiSignature};
// TODO: use MultiAddress instead of AccountId32?

//pub type Signature = AnySignature;
pub type Signature = MultiSignature;
pub type AuthorityId = <Signature as Verify>::Signer;
//pub type AccountId = MultiAddress<AccountId32,;
pub type AccountId = AccountId32;
pub type Hash = sp_core::H256;
pub type BalanceTransferFn = ([u8; 2], AccountId, Compact<u128>);
//FIXME: Is this really necessary to define all variables three times?
//pub static BALANCE_MODULE: u8 = 4u8;
//pub static BALANCE_TRANSFER: u8 = 0u8;
pub static SUBSRATEE_REGISTRY_MODULE: u8 = 8u8;
pub static UNSHIELD: u8 = 6u8;
//pub static CALL_CONFIRMED: u8 = 3u8;

pub type ShardIdentifier = H256;
//pub type Index = u32;

#[derive(Clone)]
pub enum KeyPair {
    Sr25519(sr25519::Pair),
    Ed25519(ed25519::Pair),
}

impl KeyPair {
    fn sign(&self, payload: &[u8]) -> Signature {
        match self {
            Self::Sr25519(pair) => pair.sign(payload).into(),
            Self::Ed25519(pair) => pair.sign(payload).into(),
        }
    }
}

impl From<ed25519::Pair> for KeyPair {
    fn from(x: ed25519::Pair) -> Self {
        KeyPair::Ed25519(x)
    }
}

impl From<sr25519::Pair> for KeyPair {
    fn from(x: sr25519::Pair) -> Self {
        KeyPair::Sr25519(x)
    }
}

#[cfg(feature = "sgx")]
pub mod sgx;

#[cfg(feature = "std")]
pub mod cli;

#[cfg(feature = "sgx")]
//pub type State = sp_io::SgxExternalitiesType;
pub type StateType = sgx_externalities::SgxExternalitiesType;
#[cfg(feature = "sgx")]
pub type State = sgx_externalities::SgxExternalities;
#[cfg(feature = "sgx")]
pub type StateTypeDiff = sgx_externalities::SgxExternalitiesDiffType;

#[derive(Encode, Decode, Clone, core::fmt::Debug)]
#[allow(non_camel_case_types)]
pub enum TrustedOperation {
    indirect_call(TrustedCallSigned),
    direct_call(TrustedCallSigned),
    get(Getter),
}

impl From<TrustedCallSigned> for TrustedOperation {
    fn from(item: TrustedCallSigned) -> Self {
        TrustedOperation::indirect_call(item)
    }
}

impl From<Getter> for TrustedOperation {
    fn from(item: Getter) -> Self {
        TrustedOperation::get(item)
    }
}

impl From<TrustedGetterSigned> for TrustedOperation {
    fn from(item: TrustedGetterSigned) -> Self {
        TrustedOperation::get(item.into())
    }
}

impl From<PublicGetter> for TrustedOperation {
    fn from(item: PublicGetter) -> Self {
        TrustedOperation::get(item.into())
    }
}

#[derive(Encode, Decode, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Getter {
    public(PublicGetter),
    trusted(TrustedGetterSigned),
}

impl From<PublicGetter> for Getter {
    fn from(item: PublicGetter) -> Self {
        Getter::public(item)
    }
}

impl From<TrustedGetterSigned> for Getter {
    fn from(item: TrustedGetterSigned) -> Self {
        Getter::trusted(item)
    }
}

#[derive(Encode, Decode, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum PublicGetter {
    some_value,
}

#[derive(Encode, Decode, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum TrustedCall {
    balance_set_balance(AccountId, AccountId, Balance, Balance),
    balance_transfer(AccountId, AccountId, Balance),
    balance_unshield(AccountId, AccountId, Balance, ShardIdentifier), // (AccountIncognito, BeneficiaryPublicAccount, Amount, Shard)
    balance_shield(AccountId, Balance),                               // (AccountIncognito, Amount)
}

impl TrustedCall {
    pub fn account(&self) -> &AccountId {
        match self {
            TrustedCall::balance_set_balance(account, _, _, _) => account,
            TrustedCall::balance_transfer(account, _, _) => account,
            TrustedCall::balance_unshield(account, _, _, _) => account,
            TrustedCall::balance_shield(account, _) => account,
        }
    }

    pub fn sign(
        &self,
        pair: &KeyPair,
        nonce: Index,
        mrenclave: &[u8; 32],
        shard: &ShardIdentifier,
    ) -> TrustedCallSigned {
        let mut payload = self.encode();
        payload.append(&mut nonce.encode());
        payload.append(&mut mrenclave.encode());
        payload.append(&mut shard.encode());

        TrustedCallSigned {
            call: self.clone(),
            nonce,
            signature: pair.sign(payload.as_slice()),
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum TrustedGetter {
    free_balance(AccountId),
    reserved_balance(AccountId),
    nonce(AccountId),
}

impl TrustedGetter {
    pub fn account(&self) -> &AccountId {
        match self {
            TrustedGetter::free_balance(account) => account,
            TrustedGetter::reserved_balance(account) => account,
            TrustedGetter::nonce(account) => account,
        }
    }

    pub fn sign(&self, pair: &KeyPair) -> TrustedGetterSigned {
        let signature = pair.sign(self.encode().as_slice());
        TrustedGetterSigned {
            getter: self.clone(),
            signature,
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct TrustedGetterSigned {
    pub getter: TrustedGetter,
    pub signature: Signature,
}

impl TrustedGetterSigned {
    pub fn new(getter: TrustedGetter, signature: Signature) -> Self {
        TrustedGetterSigned { getter, signature }
    }

    pub fn verify_signature(&self) -> bool {
        self.signature
            .verify(self.getter.encode().as_slice(), self.getter.account())
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct TrustedCallSigned {
    pub call: TrustedCall,
    pub nonce: Index,
    pub signature: Signature,
}

impl TrustedCallSigned {
    pub fn new(call: TrustedCall, nonce: Index, signature: Signature) -> Self {
        TrustedCallSigned {
            call,
            nonce,
            signature,
        }
    }

    pub fn verify_signature(&self, mrenclave: &[u8; 32], shard: &ShardIdentifier) -> bool {
        let mut payload = self.call.encode();
        payload.append(&mut self.nonce.encode());
        payload.append(&mut mrenclave.encode());
        payload.append(&mut shard.encode());
        self.signature
            .verify(payload.as_slice(), self.call.account())
    }

    pub fn into_trusted_operation(self, direct: bool) -> TrustedOperation {
        match direct {
            true => TrustedOperation::direct_call(self),
            false => TrustedOperation::indirect_call(self),
        }
    }
}

// TODO: #91 signed return value
/*
pub struct TrustedReturnValue<T> {
    pub value: T,
    pub signer: AccountId
}
impl TrustedReturnValue
*/

#[cfg(feature = "sgx")]
pub struct Stf {}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_keyring::AccountKeyring;

    #[test]
    fn verify_signature_works() {
        let nonce = 21;
        let mrenclave = [0u8; 32];
        let shard = ShardIdentifier::default();

        let call = TrustedCall::balance_set_balance(
            AccountKeyring::Alice.public().into(),
            AccountKeyring::Alice.public().into(),
            42,
            42,
        );
        let signed_call = call.sign(
            &KeyPair::Sr25519(AccountKeyring::Alice.pair()),
            nonce,
            &mrenclave,
            &shard,
        );

        assert!(signed_call.verify_signature(&mrenclave, &shard));
    }
}