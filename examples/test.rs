use self::some::lib::ExampleType;
use serde::{Deserialize, Serialize};

mod some {
    use super::*;
    pub mod lib {
        use super::*;
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        pub struct ExampleType(pub u32);
    }
}

struct State {
    tree: sled::Tree,
}

impl State {
    pub fn test() -> Result<Self, dbstruct::Error> {
        let db = sled::Config::default().temporary(true).open().unwrap();
        Self::open_tree(db)
    }
    pub fn open_db(
        path: impl std::convert::AsRef<std::path::Path>,
    ) -> Result<Self, dbstruct::Error> {
        let db = sled::Config::default().path(path).open().unwrap();
        Self::open_tree(db)
    }
    pub fn open_tree(db: sled::Db) -> Result<Self, dbstruct::Error> {
        let tree = db.open_tree("State,mappy").unwrap();
        Ok(Self { tree })
    }
    /// atomically push a new value into the db.
    #[allow(dead_code)]
    pub fn set_mappy(
        &self,
        key: &u8,
        value: &u16,
    ) -> std::result::Result<Option<u8>, dbstruct::Error> {
        let prefix = 0u8;
        let mut key_buffer = Vec::new();
        key_buffer.push(prefix);
        let mut key_buffer = std::io::Cursor::new(key_buffer);
        bincode::serialize_into(key_buffer, key).map_err(dbstruct::Error::Serializing)?;
        let key_bytes = key_buffer.into_inner();
        let value_bytes = bincode::serialize(value).map_err(dbstruct::Error::Serializing)?;

        let existing = match self.tree.insert(key_bytes, value_bytes)? {
            Some(bytes) => bincode::deserialize(&bytes).map_err(dbstruct::Error::DeSerializing)?,
            None => None,
        };
        Ok(existing)
    }
    /// atomically pop a new value from the db
    #[allow(dead_code)]
    pub fn get_mappy(&self, key: &u8) -> std::result::Result<Option<u16>, dbstruct::Error> {
        ::core::panicking::panic("not yet implemented")
    }
}

fn main() {
    let state = State::test().unwrap();
}
