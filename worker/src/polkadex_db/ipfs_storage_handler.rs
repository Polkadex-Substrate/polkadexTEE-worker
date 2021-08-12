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

use cid::Cid;
use log::*;
use std::convert::TryFrom;
use std::io::Cursor;
use std::sync::mpsc::channel;

use super::PolkadexDBError::IpfsError;
use super::Result;

use crate::constants::{IPFS_HOST, IPFS_PORT};

use http::uri::Scheme;
use ipfs_api::{IpfsClient, TryFromUri};

/// handles all disc permanent storage interactions of polkadex databases
pub struct IpfsStorageHandler {
    port: u16,
    host: String,
}

impl Default for IpfsStorageHandler {
    fn default() -> Self {
        IpfsStorageHandler::new(IPFS_PORT, IPFS_HOST.to_string())
    }
}

impl IpfsStorageHandler {
    fn new(port: u16, host: String) -> Self {
        IpfsStorageHandler { port, host }
    }

    #[tokio::main]
    pub async fn snapshot_to_ipfs(&mut self, data: Vec<u8>) -> Result<Cid> {
        let client = IpfsClient::from_host_and_port(Scheme::HTTP, &self.host, self.port)
           .map_err(|e| IpfsError(format!("{:?}", e)))?;

        let datac = Cursor::new(data);
        let (tx, rx) = channel();

        match client.add(datac).await {
            Ok(res) => {
                info!("Result Hash {}", res.hash);
                tx.send(res.hash)
                    .map_err(|e| IpfsError(format!("{:?}", e)))?;
            }
            Err(e) => {
                eprintln!("error adding file: {}", e);
                return Err(IpfsError(format!("{:?}", e)));
            },
        };
        let hash: &str = &rx.recv().map_err(|e| IpfsError(format!("{:?}", e)))?;
        Cid::try_from(hash).map_err(|e| IpfsError(format!("{:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;

    #[test]
    fn create_ipfs_storage_handler_works() {
        // given
        let port = 1000;
        let host = "hello".to_string();

        // when
        let handler = IpfsStorageHandler::new(port, host.clone());

        // then
        assert_eq!(handler.host, host);
        assert_eq!(handler.port, port);
    }

    // this test needs a local ipfs node running!
    #[test]
    fn snapshotting_to_ipfs_works() {
        // given
        let port = 5001;
        let host = "127.0.0.1".to_string();
        let mut handler = IpfsStorageHandler::new(port, host);
        let data = "hello_world".encode();

        // when
        let cid = handler.snapshot_to_ipfs(data).unwrap();
        println!("{:?}", cid);

        // then
        assert_eq!("QmNbwhCos4m8tK4uiLtJiCjEdj68XDMYFRdQNvAsqP3CFw", cid.to_string());
    }
}