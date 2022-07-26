use core::fmt;
use serde::{de::DeserializeOwned, Serialize};

use super::data_store::DataStore;
use super::{data_store, Error};

pub trait ByteStore {
    type Error: fmt::Debug;
    type Bytes: AsRef<[u8]>;
    fn get(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error>;
    fn remove(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error>;
    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Self::Bytes>, Self::Error>;
}

pub trait Atomic: ByteStore {
    fn atomic_update(
        &self,
        key: &[u8],
        op: impl FnMut(Option<&[u8]>) -> Option<Vec<u8>>,
    ) -> Result<(), Self::Error>;
    fn conditional_update(
        &self,
        key: &[u8],
        new: &[u8],
        expected: &[u8],
    ) -> Result<(), Self::Error>;
}

impl<K, V, E, B, BS> DataStore<K, V> for BS
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: ByteStore<Error = E, Bytes = B>,
{
    type Error = Error<E>;

    fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = BS::get(&self, &key)?;
        Ok(match val {
            Some(bytes) => bincode::deserialize(bytes.as_ref()).map_err(Error::DeSerializing)?,
            None => None,
        })
    }

    fn remove(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = BS::remove(&self, &key)?;
        Ok(match val {
            Some(bytes) => bincode::deserialize(bytes.as_ref()).map_err(Error::DeSerializing)?,
            None => None,
        })
    }

    fn insert(&self, key: &K, val: &V) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = bincode::serialize(val).map_err(Error::SerializingValue)?;
        let existing = BS::insert(&self, &key, &val)?;
        Ok(match existing {
            Some(bytes) => bincode::deserialize(bytes.as_ref()).map_err(Error::DeSerializing)?,
            None => None,
        })
    }
}

impl<K, V, E, B, BS> data_store::Atomic<K, V> for BS
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: Atomic<Error = E, Bytes = B>,
{
    fn atomic_update(
        &self,
        key: &K,
        mut op: impl FnMut(V) -> V + Clone,
    ) -> Result<(), Self::Error> {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let mut res = Ok(());
        let bytes_op = |old: Option<&[u8]>| -> Option<Vec<u8>> {
            if let Some(old) = old {
                let old = old.as_ref();
                match bincode::deserialize(old) {
                    Err(e) => {
                        res = Err(Error::DeSerializing(e));
                        Some(old.to_vec())
                    }
                    Ok(val) => {
                        let new = op(val);
                        match bincode::serialize(&new) {
                            Err(e) => {
                                res = Err(Error::DeSerializing(e));
                                Some(old.to_vec())
                            }
                            Ok(new_bytes) => Some(new_bytes),
                        }
                    }
                }
            } else {
                None
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
