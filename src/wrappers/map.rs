use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Error;

pub struct Map<K, V> {
    phantom_key: PhantomData<K>,
    phantom_val: PhantomData<V>,
    tree: sled::Tree,
    prefix: u8,
}

// Perf reuse buffer by storing it in Map (saves allocation)?
fn prefixed_key(prefix: u8, key: impl Serialize) -> Vec<u8> {
    let mut key_buffer = Vec::new();
    key_buffer.push(prefix);
    {
        let mut key_buffer = std::io::Cursor::new(&mut key_buffer[1..]);
        bincode::serialize_into(&mut key_buffer, &key)
            .map_err(Error::Serializing)
            .unwrap();
    }
    key_buffer
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Map<K, V> {
    pub fn new(tree: sled::Tree, prefix: u8) -> Self {
        Self {
            phantom_key: PhantomData,
            phantom_val: PhantomData,
            tree,
            prefix,
        }
    }

    /// returns existing value if any was set
    pub fn set(&self, key: &K, value: &V) -> Result<Option<V>, Error> {
        let mut key_buffer = Vec::new();
        key_buffer.push(self.prefix);
        let mut key_buffer = std::io::Cursor::new(&mut key_buffer[1..]);
        bincode::serialize_into(&mut key_buffer, key).map_err(Error::Serializing)?;

        let key_bytes = key_buffer.into_inner();
        let value_bytes = bincode::serialize(value).map_err(Error::Serializing)?;

        let existing = match self.tree.insert(key_bytes, value_bytes)? {
            Some(bytes) => bincode::deserialize(&bytes).map_err(Error::DeSerializing)?,
            None => None,
        };
        Ok(existing)
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, Error> {
        let mut key_buffer = Vec::new();
        key_buffer.push(self.prefix);
        let mut key_buffer = std::io::Cursor::new(&mut key_buffer[1..]);
        bincode::serialize_into(&mut key_buffer, key).map_err(Error::Serializing)?;
        let key_bytes = key_buffer.into_inner();

        let value = match self.tree.get(key_bytes)? {
            Some(bytes) => bincode::deserialize(&bytes).map_err(Error::DeSerializing)?,
            None => None,
        };
        Ok(value)
    }
}
