use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Error;

fn idx_key(idx: u64, prefix: u8) -> [u8; 9] {
    let mut res = [0u8; 9];
    res[0] = prefix;
    res[1..].copy_from_slice(&idx.to_be_bytes());
    res
}

pub struct Vec<T> {
    phantom: PhantomData<T>,
    tree: sled::Tree,
    prefix: u8,
}

impl<T: Serialize + DeserializeOwned> Vec<T> {
    pub fn new(tree: sled::Tree, prefix: u8) -> Self {
        Self {
            phantom: PhantomData,
            tree,
            prefix,
        }
    }

    pub fn push(&self, value: T) -> Result<(), Error> {
        let last_idx = self
            .tree
            .get_lt([self.prefix + 1])?
            .map(|(key, _)| {
                u64::from_be_bytes(
                    key[1..]
                        .try_into()
                        .expect("vector keys need to be prefix + valid u64 as be bytes"),
                )
            })
            .unwrap_or(0);
        let key: [u8; 9] = idx_key(last_idx + 1, self.prefix);
        let bytes = bincode::serialize(&value).map_err(Error::Serializing)?;
        self.tree.insert(key, bytes)?;
        Ok(())
    }

    pub fn pop(&self) -> Result<Option<T>, Error> {
        let last_element = match self.tree.get_lt([self.prefix + 1])?.map(|(key, _)| key) {
            Some(key) => key,
            None => return Ok(None),
        };

        let bytes = match self.tree.remove(last_element)? {
            Some(bytes) => bytes,
            None => return Ok(None), // value must been deleted between fetching len and this
        };
        let value = bincode::deserialize(&bytes).map_err(Error::DeSerializing)?;
        Ok(Some(value))
    }
}
