use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Error;

pub struct DefaultTrait<T> {
    phantom: PhantomData<T>,
    tree: sled::Tree,
    key: u8,
}

impl<T> DefaultTrait<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    pub fn new(tree: sled::Tree, key: u8) -> Self {
        Self {
            phantom: PhantomData::default(),
            tree,
            key,
        }
    }

    pub fn set(&mut self, value: &T) -> Result<(), Error> {
        let bytes = bincode::serialize(value).map_err(Error::Serializing)?;
        self.tree.insert([self.key], bytes)?;
        Ok(())
    }

    pub fn get(&self) -> Result<Option<T>, Error> {
        match self.tree.get([self.key])? {
            Some(bytes) => Ok(bincode::deserialize(&bytes).map_err(Error::DeSerializing)?),
            None => Ok(Default::default()),
        }
    }
}
