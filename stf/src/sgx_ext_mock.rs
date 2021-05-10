#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate sgx_tstd as std;

use std::{collections::HashMap, vec::Vec};

pub type SgxExternalitiesType = HashMap<Vec<u8>, Vec<u8>>;
pub type SgxExternalitiesDiffType = HashMap<Vec<u8>, Option<Vec<u8>>>;

#[cfg_attr(not(feature = "std"), derive(Debug, Clone))]
pub struct SgxExternalities {
    pub state: SgxExternalitiesType,
    pub state_diff: SgxExternalitiesDiffType,
}

pub trait SgxExternalitiesTrait {
    fn new() -> Self;
    fn decode(state: Vec<u8>) -> Self;
    fn encode(self) -> Vec<u8>;
}

pub trait SgxExternalitiesTypeTrait {
    fn new() -> Self;
    fn decode(state: Vec<u8>) -> Self;
    fn encode(self) -> Vec<u8>;
}

#[cfg(not(feature = "std"))]
impl SgxExternalitiesTypeTrait for SgxExternalitiesType {
    fn new() -> Self {
            Default::default()
    }
    fn decode(state: Vec<u8>) -> Self {
        Default::default()
    }

    fn encode(self) -> Vec<u8> {
        Vec::new()
    }
}

#[cfg(not(feature = "std"))]
impl SgxExternalitiesTypeTrait for SgxExternalitiesDiffType {
    fn new() -> Self {
            Default::default()
    }
    fn decode(state: Vec<u8>) -> Self {
        Default::default()
    }

    fn encode(self) -> Vec<u8> {
        Vec::new()
    }
}

#[cfg(not(feature = "std"))]
impl SgxExternalitiesTrait for SgxExternalities {
    /// Create a new instance of `BasicExternalities`
    fn new() -> Self {
        SgxExternalities{
            state: SgxExternalitiesType::new(),
            state_diff:SgxExternalitiesDiffType::new(),
        }
    }

    fn decode(state: Vec<u8>) -> Self {
        Self::new()
    }

    fn encode(self) -> Vec<u8> {
        Vec::new()
    }
}