#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

pub mod accounts;
pub mod types;

pub extern crate alloc;
use alloc::string::{String, ToString};

use codec::{Decode, Encode};
pub use polkadex_primitives::common_types::{AccountId, Balance, Signature};
use sp_core::{H256};
use sp_std::vec::Vec;
pub use polkadex_primitives::assets::AssetId;

pub type ShardIdentifier = H256;
pub type BlockNumber = u32;

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct LinkedAccount {
    pub prev: AccountId,
    pub current: AccountId,
    pub next: Option<AccountId>,
    pub proxies: Vec<AccountId>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct PolkadexAccount {
    pub account: LinkedAccount,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct OpenFinexUri {
    ip: Vec<u8>,
    port: u16,
    path: Vec<u8>
}

impl OpenFinexUri {
    /// creates a new openfinex uri used to connect to openfinex server.
    /// INPUT:  ip_str (e.g. "localhost" or "127.0.0.1")
    ///         port_path_str, string containing at least the port
    ///             (e.g.: 8001/api/v2/ws or 8001)
    pub fn new(ip_str: &str, port_path_str: &str) -> Self {
        let (port_str, path_str)  = if let Some(index) = port_path_str.find("/") {
            port_path_str.split_at(index)
        } else {
            (port_path_str, "/")
        };
        OpenFinexUri{
            ip: ip_str.as_bytes().to_vec(),
            port: port_str.parse().unwrap(),
            path: path_str.as_bytes().to_vec(),
        }
    }

    //FIXME: we should use &str or anything similar here..
    pub fn ip(&self) -> String {
        String::from_utf8_lossy(&self.ip).into_owned()
    }

    pub fn port(&self) -> String {
        self.port.to_string()
    }

    // FIXME: maybe port should be a u XX right away?
    pub fn port_u16(&self) -> u16 {
        self.port
    }

    pub fn path(&self) -> String {
        String::from_utf8_lossy(&self.path).into_owned()
   }
}

// TODO: Add unit tests