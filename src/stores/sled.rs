use serde::{de::DeserializeOwned, Serialize};

use crate::traits::DataStore;

pub struct Sled {
    tree: sled::Tree,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("value could not be deserialized using bincode")]
    DeSerializing(bincode::Error),
    #[error("value could not be serialized using bincode")]
    SerializingValue(bincode::Error),
    #[error("could not serialize key using bincode")]
    SerializingKey(bincode::Error),
    #[error("the database returned an error")]
    Database(#[from] sled::Error),
}

impl<K, V> DataStore<K, V> for Sled
where
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
    type Error = Error;

    fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        match self.tree.get(key)? {
            Some(bytes) => Ok(bincode::deserialize(&bytes).map_err(Error::DeSerializing)?),
            None => Ok(None),
        }
    }

    fn remove<'a>(&self, key: &'a K) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let existing = self.tree.remove(&key)?;
        match existing {
            None => Ok(None),
            Some(bytes) => {
                let val = bincode::deserialize(&bytes).map_err(Error::DeSerializing)?;
                Ok(Some(val))
            }
        }
    }

    fn insert<'a>(&self, key: &'a K, val: &'a V) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = bincode::serialize(val).map_err(Error::SerializingValue)?;
        let existing = self.tree.insert(key, val)?;

        match existing {
            None => Ok(None),
            Some(bytes) => {
                let val = bincode::deserialize(&bytes).map_err(Error::DeSerializing)?;
                Ok(Some(val))
            }
        }
    }

    fn atomic_update(&self, key: &K, op: impl FnMut(V) -> V + Clone) -> Result<(), Self::Error> {
        let mut res = Ok(());
        let update = |old: Option<&[u8]>| match old {
            None => None,
            Some(old) => match bincode::deserialize(old) {
                Err(e) => {
                    res = Err(Error::DeSerializing(e));
                    Some(old.to_vec())
                }
                Ok(v) => {
                    let new = op.clone()(v);
                    match bincode::serialize(&new) {
                        Ok(new_bytes) => Some(new_bytes),
                        Err(e) => {
                            res = Err(Error::SerializingValue(e));
                            Some(old.to_vec())
                        }
                    }
                }
            },
        };

        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        self.tree.update_and_fetch(self.key, update)?;
        Ok(())
    }

    fn conditional_update(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error> {
        todo!()
    }
}
