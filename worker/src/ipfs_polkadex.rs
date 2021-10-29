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

use std::io::Cursor;
use std::sync::mpsc::channel;

use futures::TryStreamExt;
use ipfs_api::IpfsClient;
use log::*;

use cid::multibase::Base;
pub use cid::Cid;
use std::convert::TryFrom;

#[tokio::main]
async fn _write_to_ipfs(data: &'static [u8]) -> Result<Cid, String> {
    // Creates an `IpfsClient` connected to the endpoint specified in ~/.ipfs/api.
    // If not found, tries to connect to `localhost:5001`.
    let client = IpfsClient::default();

    match client.version().await {
        Ok(version) => info!("version: {:?}", version.version),
        Err(e) => eprintln!("error getting version: {}", e),
    }

    let datac = Cursor::new(data);
    let (tx, rx) = channel();

    match client.add(datac).await {
        Ok(res) => {
            info!("Result Hash {}", res.hash);
            tx.send(res.hash.into_bytes()).unwrap();
        }
        Err(e) => eprintln!("error adding file: {}", e),
    }

    Cid::try_from(
        rx.recv()
            .map_err(|_| String::from("Failed to receive CID"))?,
    )
    .map_err(|_| String::from("Failed to build CID"))
}

#[tokio::main]
pub async fn read_from_ipfs(cid: Cid) -> Result<Vec<u8>, String> {
    // Creates an `IpfsClient` connected to the endpoint specified in ~/.ipfs/api.
    // If not found, tries to connect to `localhost:5001`.
    let client = IpfsClient::default();

    let h = cid.to_string_of_base(Base::Base58Btc).map_err(|_| {
        String::from("CID couldn't be converted to the correct base, check if the CID is correct")
    })?;

    info!("Fetching content from: {}", h);

    client
        .cat(h.as_str())
        .map_ok(|chunk| chunk.to_vec())
        .map_err(|e| e.to_string())
        .try_concat()
        .await
}
