#![allow(clippy::type_complexity)]
//! Helper traits that are easier to implement. These implement the similarly named trait in
//! [`data_store`].

use core::fmt;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::ops::RangeBounds;
use tracing::{instrument, trace};

use super::byte_store;
use super::data_store;
use super::data_store::DataStore;
use crate::Error;

/// A helper trait, implementing this automatically implements
/// [`DataStore`][super::data_store::DataStore]
pub trait ByteStore {
    type DbError: fmt::Debug;
    type Bytes: AsRef<[u8]>;
    fn get(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError>;
    fn remove(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError>;
    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError>;
}

/// A helper trait, implementing this automatically implements
/// [`data_store::Atomic`][super::data_store::Atomic]
pub trait Atomic: ByteStore {
    fn atomic_update(
        &self,
        key: &[u8],
        op: impl FnMut(Option<&[u8]>) -> Option<Vec<u8>>,
    ) -> Result<(), Self::DbError>;
    fn conditional_update(
        &self,
        key: &[u8],
        new: &[u8],
        expected: &[u8],
    ) -> Result<(), Self::DbError>;
}

/// A helper trait, implementing this automatically implements
/// [`data_store::Ordered`][super::data_store::Ordered]
pub trait Ordered: ByteStore {
    /// returns the previous key value pair before key
    fn get_lt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::DbError>;
    /// returns the next key value pair after key
    fn get_gt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::DbError>;
}

/// A helper trait, implementing this automatically implements
/// [`data_store::Ranged`][super::data_store::Ranged]
pub trait Ranged: Ordered {
    type Key: AsRef<[u8]>;
    type Iter: Iterator<Item = Result<(Self::Bytes, Self::Bytes), Self::DbError>>;
    /// returns the previous key value pair before key
    fn range(&self, range: impl RangeBounds<Self::Key>) -> Self::Iter;
}

impl<E, B, BS> DataStore for BS
where
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: ByteStore<DbError = E, Bytes = B>,
{
    type DbError = Error<E>;

    #[instrument(skip_all, level = "trace", err)]
    fn get<K, V>(&self, key: &K) -> Result<Option<V>, Error<Self::DbError>>
    where
        K: Serialize,
        V: DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::<Self::DbError>::SerializingKey)?;
        trace!("getting value for key: {key:?}");
        let val = BS::get(self, &key).map_err(Error::Database)?;
        Ok(match val {
            Some(bytes) => {
                trace!("bytes of value: {:?}", bytes.as_ref());
                let val = bincode::deserialize(bytes.as_ref())
                    .map_err(Error::<Self::DbError>::DeSerializingVal)?;
                Some(val)
            }
            None => None,
        })
    }

    #[instrument(skip_all, level = "trace", err)]
    fn remove<K, V>(&self, key: &K) -> Result<Option<V>, Error<Self::DbError>>
    where
        K: Serialize,
        V: DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::<Self::DbError>::SerializingKey)?;
        trace!("removing at key: {key:?}");
        let val = BS::remove(self, &key).map_err(Error::Database)?;
        Ok(match val {
            Some(bytes) => {
                trace!("bytes of current value: {:?}", bytes.as_ref());
                let val = bincode::deserialize(bytes.as_ref())
                    .map_err(Error::<Self::DbError>::DeSerializingVal)?;
                Some(val)
            }
            None => None,
        })
    }

    #[instrument(skip_all, level = "trace", err)]
    fn insert<K, V>(&self, key: &K, val: &V) -> Result<Option<V>, Error<Self::DbError>>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::<Self::DbError>::SerializingKey)?;
        let val = bincode::serialize(val).map_err(Error::<Self::DbError>::SerializingValue)?;
        trace!("inserting key: {key:?}, val: {val:?}");
        let existing = BS::insert(self, &key, &val).map_err(Error::Database)?;
        Ok(match existing {
            Some(bytes) => {
                trace!("bytes of previous value: {:?}", bytes.as_ref());
                trace!("deserializing to: {}", std::any::type_name::<V>());
                Some(
                    bincode::deserialize(bytes.as_ref())
                        .map_err(Error::<Self::DbError>::DeSerializingVal)?,
                )
            }
            None => None,
        })
    }
}

impl<E, B, BS> data_store::Atomic for BS
where
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: Atomic<DbError = E, Bytes = B>,
{
    #[instrument(skip_all, level = "trace", err)]
    fn atomic_update<K, V>(
        &self,
        key: &K,
        mut op: impl FnMut(V) -> V + Clone,
    ) -> Result<(), Self::DbError>
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
                        res = Err(Error::DeSerializingVal(e));
                        Some(old.to_vec())
                    }
                    Ok(val) => {
                        let new = op(val);
                        match bincode::serialize(&new) {
                            Err(e) => {
                                res = Err(Error::DeSerializingVal(e));
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
    fn conditional_update<K, V>(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::DbError>
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

impl<E, B, BS> data_store::Ordered for BS
where
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: byte_store::Ordered<DbError = E, Bytes = B>,
{
    #[instrument(skip_all, level = "trace", err)]
    fn get_lt<InKey, OutKey, Value>(
        &self,
        key: &InKey,
    ) -> Result<Option<(OutKey, Value)>, Self::DbError>
    where
        InKey: Serialize,
        OutKey: Serialize + DeserializeOwned,
        Value: Serialize + DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        trace!("getting less then key: {key:?}");
        Ok(match byte_store::Ordered::get_lt(self, &key)? {
            None => None,
            Some((key, val)) => {
                trace!(
                    "key ({}): {:?}, val ({}): {:?}",
                    std::any::type_name::<OutKey>(),
                    key.as_ref(),
                    std::any::type_name::<dyn Value>(),
                    val.as_ref()
                );
                let key = bincode::deserialize(key.as_ref()).map_err(Error::DeSerializingKey)?;
                let val = bincode::deserialize(val.as_ref()).map_err(Error::DeSerializingVal)?;
                Some((key, val))
            }
        })
    }

    #[instrument(skip_all, level = "trace", err)]
    fn get_gt<InKey, OutKey, Value>(
        &self,
        key: &InKey,
    ) -> Result<Option<(OutKey, Value)>, Self::DbError>
    where
        InKey: Serialize,
        OutKey: Serialize + DeserializeOwned,
        Value: Serialize + DeserializeOwned,
    {
        let key = bincode::serialize(key).map_err(Error::SerializingKey)?;
        trace!("getting greater then key: {key:?}");
        Ok(match byte_store::Ordered::get_gt(self, &key)? {
            None => None,
            Some((key, val)) => {
                trace!(
                    "key ({}): {:?}, val ({}): {:?}",
                    std::any::type_name::<OutKey>(),
                    key.as_ref(),
                    std::any::type_name::<dyn Value>(),
                    val.as_ref()
                );
                let key = bincode::deserialize(key.as_ref()).map_err(Error::DeSerializingKey)?;
                let val = bincode::deserialize(val.as_ref()).map_err(Error::DeSerializingVal)?;
                Some((key, val))
            }
        })
    }
}

struct IterWrapper<I, OutKey, Value, Bytes, Error> {
    iter: I,
    key_phantom: PhantomData<OutKey>,
    val_phantom: PhantomData<Value>,
    bytes_phantom: PhantomData<Bytes>,
    error_phantom: PhantomData<Error>,
}

impl<OutKey, Value, Bytes, E, I> Iterator for IterWrapper<I, OutKey, Value, Bytes, E>
where
    E: fmt::Debug,
    Bytes: AsRef<[u8]>,
    OutKey: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    I: Iterator<Item = Result<(Bytes, Bytes), E>>,
{
    type Item = Result<(OutKey, Value), Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|res| match res {
            Ok((key, val)) => bincode::deserialize(key.as_ref())
                .map_err(Error::DeSerializingKey)
                .and_then(|key| {
                    bincode::deserialize(val.as_ref())
                        .map_err(Error::DeSerializingVal)
                        .map(|val| (key, val))
                }),
            Err(e) => Err(Error::Database(e)),
        })
    }
}

impl<E, B, BS> data_store::Ranged for BS
where
    E: fmt::Debug,
    B: AsRef<[u8]>,
    BS: byte_store::Ranged<DbError = E, Bytes = B, Key = Vec<u8>>,
{
    fn range<InKey, OutKey, Value>(
        &self,
        range: impl RangeBounds<InKey>,
    ) -> Result<impl Iterator<Item = Result<(OutKey, Value), Self::DbError>>, Self::DbError>
    where
        InKey: Serialize,
        OutKey: Serialize + DeserializeOwned,
        Value: Serialize + DeserializeOwned,
    {
        use std::ops::Bound;
        let start_bound = match range.start_bound() {
            Bound::Included(key) => {
                Bound::Included(bincode::serialize(key).map_err(Error::SerializingKey)?)
            }
            Bound::Excluded(key) => {
                Bound::Excluded(bincode::serialize(key).map_err(Error::SerializingKey)?)
            }
            Bound::Unbounded => Bound::Unbounded,
        };
        let end_bound = match range.end_bound() {
            Bound::Included(key) => {
                Bound::Included(bincode::serialize(key).map_err(Error::SerializingKey)?)
            }
            Bound::Excluded(key) => {
                Bound::Excluded(bincode::serialize(key).map_err(Error::SerializingKey)?)
            }
            Bound::Unbounded => Bound::Unbounded,
        };

        let iter = byte_store::Ranged::range(self, (start_bound, end_bound));
        Ok(IterWrapper {
            iter,
            key_phantom: PhantomData,
            val_phantom: PhantomData,
            bytes_phantom: PhantomData::<B>,
            error_phantom: PhantomData,
        })
    }
}
