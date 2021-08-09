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

use std::collections::HashMap;
use codec::{Encode, Decode};
use super::PermanentStorageHandler;
use super::Result;
use super::PolkadexDBError as Error;

pub type EncodableDB = Vec<(Vec<u8>, Vec<u8>)>;
#[derive(Debug)]
pub struct GeneralDB<D: PermanentStorageHandler> {
    pub db: HashMap<Vec<u8>, Vec<u8>>,
    pub disc_storage: D
}

impl<D: PermanentStorageHandler> GeneralDB<D> {
    pub fn write(&mut self, key: Vec<u8>, data: Vec<u8>) {
        self.db.insert(key, data);
    }

    pub fn _find(&self, k: Vec<u8>) -> Option<&Vec<u8>> {
        self.db.get(&k)
    }

    pub fn _delete(&mut self, k: Vec<u8>) {
        self.db.remove(&k);
    }

    /// reads from memory
    pub fn read_all(&self) -> EncodableDB {
        self.db
            .clone()
            .into_iter()
            .collect::<Vec<(Vec<u8>, Vec<u8>)>>()
    }

    /// writes from memory to permanent disc storage
    pub fn write_disk(&self) -> Result<()> {
        self.disc_storage.write_to_storage(
            &self.read_all()
                .encode()
                .as_slice()
        )
    }

    /// reads from permanent disc storage to memory
    pub fn read_disk(&mut self) -> Result<()> {
        let data = EncodableDB::decode(
            &mut self.disc_storage.read_from_storage()?
            .as_slice()
        ).map_err(Error::DecodeError)?;
        for data_point in self.db.clone() {
            self.write(data_point.0, data_point.1);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::GeneralDB;
    use codec::Encode;
    use std::collections::HashMap;
    use super::mock::PermanentStorageMock;

    #[test]
    fn write() {
        let mut general_db = GeneralDB { db: HashMap::new(), disc_storage: PermanentStorageMock::default() };
        assert_eq!(general_db.db, HashMap::new());
        general_db.write("key".encode(), "data".encode());
        assert_eq!(general_db.db.get(&"key".encode()), Some(&"data".encode()));
    }

    #[test]
    fn find() {
        let mut general_db = GeneralDB { db: HashMap::new() };
        general_db.db.insert("key".encode(), "data".encode());
        assert_eq!(general_db._find("key".encode()), Some(&"data".encode()));
        assert_eq!(general_db._find("key1".encode()), None);
    }

    #[test]
    fn delete() {
        let mut general_db = GeneralDB { db: HashMap::new() };
        general_db.db.insert("key".encode(), "data".encode());
        assert_eq!(general_db.db.contains_key(&"key".encode()), true);
        general_db._delete("key".encode());
        assert_eq!(general_db.db.contains_key(&"key".encode()), false);
    }

    #[test]
    fn read_all() {
        let mut general_db = GeneralDB { db: HashMap::new() };
        general_db.db.insert("key".encode(), "data".encode());
        general_db.db.insert("key1".encode(), "data1".encode());
        assert_eq!(
            {
                let mut read_all = general_db.read_all();
                read_all.sort();
                read_all
            },
            vec![
                ("key".encode(), "data".encode()),
                ("key1".encode(), "data1".encode())
            ]
        );
    }
}
