use core::fmt;
use serde::{de::DeserializeOwned, Serialize};

pub trait DataStore<K, V>
where
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
    type Error: fmt::Debug;
    fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    fn remove<'a>(&self, key: &'a K) -> Result<Option<V>, Self::Error>;
    fn insert<'a>(&self, key: &'a K, val: &'a V) -> Result<Option<V>, Self::Error>;
    /// on error the update is aborted
    fn atomic_update(&self, key: &K, op: impl FnMut(V) -> V + Clone) -> Result<(), Self::Error>;
    fn conditional_update(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error>;
}

trait BytesStore {
    type Error: fmt::Debug;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn remove(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn atomic_update(
        &self,
        key: &[u8],
        op: impl FnMut(&[u8]) -> Vec<u8>,
    ) -> Result<(), Self::Error>;
    fn conditional_update(
        &self,
        key: &[u8],
        new: &[u8],
        expected: &[u8],
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error<DbError: fmt::Debug> {
    #[error("value could not be deserialized using bincode")]
    DeSerializing(bincode::Error),
    #[error("value could not be serialized using bincode")]
    SerializingValue(bincode::Error),
    #[error("could not serialize key using bincode")]
    SerializingKey(bincode::Error),
    #[error("the database returned an error")]
    Database(#[from] DbError),
}

impl<K, V, E, BS> DataStore<K, V> for BS
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    BS: BytesStore<Error = E>,
{
    type Error = Error<E>;

    fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = BS::get(&self, &key)?;
        Ok(match val {
            Some(bytes) => bincode::deserialize(&bytes).map_err(Error::DeSerializing)?,
            None => None,
        })
    }

    fn remove(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = BS::remove(&self, &key)?;
        Ok(match val {
            Some(bytes) => bincode::deserialize(&bytes).map_err(Error::DeSerializing)?,
            None => None,
        })
    }

    fn insert(&self, key: &K, val: &V) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = bincode::serialize(val).map_err(Error::SerializingValue)?;
        let existing = BS::insert(&self, &key, &val)?;
        Ok(match existing {
            Some(bytes) => bincode::deserialize(&bytes).map_err(Error::DeSerializing)?,
            None => None,
        })
    }

    fn atomic_update(
        &self,
        key: &K,
        mut op: impl FnMut(V) -> V + Clone,
    ) -> Result<(), Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let mut res = Ok(());
        let bytes_op = |old: &[u8]| -> Vec<u8> {
            match bincode::deserialize(old) {
                Err(e) => {
                    res = Err(Error::DeSerializing(e));
                    old.to_vec()
                }
                Ok(val) => {
                    let new = op(val);
                    match bincode::serialize(&new) {
                        Err(e) => {
                            res = Err(Error::DeSerializing(e));
                            old.to_vec()
                        }
                        Ok(new_bytes) => new_bytes,
                    }
                }
            }
        };
        BS::atomic_update(&self, &key, bytes_op)?;
        res
    }

    fn conditional_update(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let new = bincode::serialize(new).map_err(Error::SerializingValue)?;
        let expected = bincode::serialize(expected).map_err(Error::SerializingValue)?;
        BS::conditional_update(&self, &key, &new, &expected)?;
        Ok(())
    }
}
