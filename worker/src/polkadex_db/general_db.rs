use std::collections::HashMap;

pub struct GeneralDB {
    pub db: HashMap<Vec<u8>, Vec<u8>>,
}

impl GeneralDB {
    pub fn write(&mut self, key: Vec<u8>, data: Vec<u8>) {
        self.db.insert(key, data);
    }

    pub fn _find(&self, k: Vec<u8>) -> Option<&Vec<u8>> {
        self.db.get(&k)
    }

    pub fn _delete(&mut self, k: Vec<u8>) {
        self.db.remove(&k);
    }

    pub fn read_all(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.db
            .clone()
            .into_iter()
            .collect::<Vec<(Vec<u8>, Vec<u8>)>>()
    }
}

#[cfg(test)]
mod tests {
    use super::GeneralDB;
    use codec::Encode;
    use std::collections::HashMap;

    #[test]
    fn write() {
        let mut general_db = GeneralDB { db: HashMap::new() };
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
