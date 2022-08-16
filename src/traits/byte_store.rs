use core::fmt;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{instrument, trace};

use super::data_store;
use super::data_store::DataStore;
use crate::Error;

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

impl<E, B, BS> DataStore for BS
where
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: ByteStore<Error = E, Bytes = B>,
{
    type Error = Error<E>;

    #[instrument(skip_all, level = "trace", err)]
    fn get<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        K: Serialize,
        V: DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        trace!("getting value for key: {key:?}");
        let val = BS::get(self, &key)?;
        Ok(match val {
            Some(bytes) => {
                trace!("bytes of value: {:?}", bytes.as_ref());
                let val = bincode::deserialize(bytes.as_ref()).map_err(Error::DeSerializing)?;
                Some(val)
            }
            None => None,
        })
    }

    #[instrument(skip_all, level = "trace", err)]
    fn remove<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        K: Serialize,
        V: DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        trace!("removing at key: {key:?}");
        let val = BS::remove(self, &key)?;
        Ok(match val {
            Some(bytes) => {
                trace!("bytes of current value: {:?}", bytes.as_ref());
                let val = bincode::deserialize(bytes.as_ref()).map_err(Error::DeSerializing)?;
                Some(val)
            }
            None => None,
        })
    }

    #[instrument(skip_all, level = "trace", err)]
    fn insert<K, V>(&self, key: &K, val: &V) -> Result<Option<V>, Self::Error>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let val = bincode::serialize(val).map_err(Error::SerializingValue)?;
        trace!("inserting key: {key:?}, val: {val:?}");
        let existing = BS::insert(self, &key, &val)?;
        Ok(match existing {
            Some(bytes) => {
                trace!("bytes of previous value: {:?}", bytes.as_ref());
                trace!("deserializing to: {}", std::any::type_name::<V>());
                Some(bincode::deserialize(bytes.as_ref()).map_err(Error::DeSerializing)?)
            }
            None => None,
        })
    }
}

impl<E, B, BS> data_store::Atomic for BS
where
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: Atomic<Error = E, Bytes = B>,
{
    #[instrument(skip_all, level = "trace", err)]
    fn atomic_update<K, V>(
        &self,
        key: &K,
        mut op: impl FnMut(V) -> V + Clone,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let mut res = Ok(());
        let bytes_op = |old: Option<&[u8]>| -> Option<Vec<u8>> {
            if let Some(old) = old {
                trace!("bytes of current value: {old:?}");
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
        BS::atomic_update(self, &key, bytes_op)?;
        res
    }

    #[instrument(skip_all, level = "trace", err)]
    fn conditional_update<K, V>(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        let new = bincode::serialize(new).map_err(Error::SerializingValue)?;
        let expected = bincode::serialize(expected).map_err(Error::SerializingValue)?;
        BS::conditional_update(self, &key, &new, &expected)?;
        Ok(())
    }
}
