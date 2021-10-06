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

use super::PermanentStorageHandler;
use super::PolkadexDBError as Error;
use super::Result;
use codec::{Decode, Encode};
use std::collections::HashMap;

pub type EncodableDB = Vec<(Vec<u8>, Vec<u8>)>;
#[derive(Debug)]
pub struct GeneralDB<D: PermanentStorageHandler> {
    pub db: HashMap<Vec<u8>, Vec<u8>>,
    pub disk_storage: D,
}

impl<D: PermanentStorageHandler> GeneralDB<D> {
    pub fn new(db: HashMap<Vec<u8>, Vec<u8>>, disk_storage: D) -> Self {
        GeneralDB { db, disk_storage }
    }
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
        self.db.clone().into_iter().collect::<EncodableDB>()
    }

    /// writes from memory to permanent disc storage
    /// FIXME: Should be signed by enclave! (issue #15)
    pub fn write_disk_from_memory(&mut self) -> Result<Vec<u8>> {
        let encoded_data = self.read_all().encode();
        self.disk_storage
            .write_to_storage(&encoded_data.as_slice())?;
        Ok(encoded_data)
    }

    #[allow(dead_code)]
    pub fn write_data_to_disk(&mut self, data: Vec<u8>) -> Result<()> {
        self.disk_storage.write_to_storage(&data.as_slice())?;
        Ok(())
    }

    /// reads from permanent disc storage to memory
    /// FIXME: Should be signed by enclave! (issue #15)
    #[allow(unused)]
    pub fn read_disk_into_memory(&mut self) -> Result<EncodableDB> {
        let data = EncodableDB::decode(&mut self.disk_storage.read_from_storage()?.as_slice())
            .map_err(Error::DecodeError)?;
        for data_point in data.clone() {
            self.write(data_point.0, data_point.1);
        }
        Ok(data)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::polkadex_db::mock::PermanentStorageMock;
    use codec::Encode;
    use std::collections::HashMap;

    #[test]
    fn write() {
        let mut general_db = GeneralDB::new(HashMap::new(), PermanentStorageMock::default());
        assert_eq!(general_db.db, HashMap::new());
        general_db.write("key".encode(), "data".encode());
        assert_eq!(general_db.db.get(&"key".encode()), Some(&"data".encode()));
    }

    #[test]
    fn find() {
        let mut general_db = GeneralDB::new(HashMap::new(), PermanentStorageMock::default());
        general_db.db.insert("key".encode(), "data".encode());
        assert_eq!(general_db._find("key".encode()), Some(&"data".encode()));
        assert_eq!(general_db._find("key1".encode()), None);
    }

    #[test]
    fn delete() {
        let mut general_db = GeneralDB::new(HashMap::new(), PermanentStorageMock::default());
        general_db.db.insert("key".encode(), "data".encode());
        assert!(general_db.db.contains_key(&"key".encode()));
        general_db._delete("key".encode());
        assert!(!general_db.db.contains_key(&"key".encode()));
    }

    #[test]
    fn read_all() {
        let mut general_db = GeneralDB::new(HashMap::new(), PermanentStorageMock::default());
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

    #[test]
    fn encodeable_db_encode_decode_roundabout_works() {
        // given
        let (key_one, entry_one) = ("key_one".encode(), "Oh wow, I'm being written!".encode());
        let (key_two, entry_two) = ("key_two".encode(), "Congrats....".encode());
        let (key_three, entry_three) = ("key_three".encode(), "Mee too, me too".encode());
        let vector: EncodableDB = vec![
            (key_one, entry_one),
            (key_two, entry_two),
            (key_three, entry_three),
        ];

        // when
        let vector_encoded = vector.encode();

        // then
        let decoded_vector = EncodableDB::decode(&mut vector_encoded.as_slice()).unwrap();
        assert_eq!(vector, decoded_vector);
    }

    #[test]
    fn write_disk_writes_all_data() {
        // given
        let entry_one = ("key_one".encode(), "Oh wow, I'm being written!".encode());
        let entry_two = ("key_two".encode(), "Congrats....".encode());
        let entry_three = ("key_three".encode(), "Mee too, me too".encode());
        let mut map = HashMap::new();
        map.insert(entry_one.0.clone(), entry_one.1.clone());
        map.insert(entry_two.0.clone(), entry_two.1.clone());
        map.insert(entry_three.0.clone(), entry_three.1.clone());
        let mut general_db = GeneralDB::new(map, PermanentStorageMock::default());

        // when
        general_db.write_disk_from_memory().unwrap();

        // then
        let contains =
            EncodableDB::decode(&mut general_db.disk_storage.contained_data.as_slice()).unwrap();
        assert!(contains.contains(&entry_one));
        assert!(contains.contains(&entry_two));
        assert!(contains.contains(&entry_three));
    }

    #[test]
    fn read_disk_returns_all_data() {
        // given
        let (key_one, entry_one) = ("key_one".encode(), "Oh wow, I'm being written!".encode());
        let (key_two, entry_two) = ("key_two".encode(), "Congrats....".encode());
        let (key_three, entry_three) = ("key_three".encode(), "Mee too, me too".encode());
        let assosciated_vector: EncodableDB = vec![
            (key_one, entry_one),
            (key_two, entry_two),
            (key_three, entry_three),
        ];
        let mut general_db = GeneralDB::new(HashMap::new(), PermanentStorageMock::default());

        general_db.disk_storage.contained_data = assosciated_vector.encode();

        // when
        let read_data = general_db.read_disk_into_memory().unwrap();

        // then
        assert_eq!(assosciated_vector, read_data);
    }

    #[test]
    fn read_disk_reads_and_stores_all_data() {
        // given
        // empty memory db
        let mut general_db = GeneralDB::new(HashMap::new(), PermanentStorageMock::default());
        // store entry in permanent disk storage
        let (key_one, entry_one) = ("key_one".encode(), "Oh wow, I'm being written!".encode());
        let (key_two, entry_two) = ("key_two".encode(), "Congrats....".encode());
        let (key_three, entry_three) = ("key_three".encode(), "Mee too, me too".encode());
        let mut map = HashMap::new();
        map.insert(key_one.clone(), entry_one.clone());
        map.insert(key_two.clone(), entry_two.clone());
        map.insert(key_three.clone(), entry_three.clone());
        let assosciated_vector: EncodableDB = vec![
            (key_one, entry_one),
            (key_two, entry_two),
            (key_three, entry_three),
        ];
        general_db.disk_storage.contained_data = assosciated_vector.encode();

        // when
        general_db.read_disk_into_memory().unwrap();

        // then
        assert_eq!(general_db.db, map);
    }
}
