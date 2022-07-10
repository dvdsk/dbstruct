use core::fmt;
use serde::{de::DeserializeOwned, Serialize};

pub trait DataStore<K, V>
where
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
    type Error: fmt::Debug;
    fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    fn insert<'a>(&self, key: &'a K, val: &'a V) -> Result<Option<V>, Self::Error>;
    fn atomic_update(&self, key: &K, op: impl FnMut(V) -> V + Clone) -> Result<(), Self::Error>;
    fn conditional_update(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error>;
}

trait BytesStore {
    type Error: fmt::Debug;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
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

impl<K, V, E, BS> DataStore<K, V> for BS
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    BS: BytesStore<Error = E>,
{
    type Error = E;

    fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).unwrap();
        let val = BS::get(&self, &key).unwrap();
        Ok(match val {
            Some(bytes) => bincode::deserialize(&bytes).unwrap(),
            None => None,
        })
    }

    fn insert(&self, key: &K, val: &V) -> Result<Option<V>, Self::Error> {
        let key = bincode::serialize(key).unwrap();
        let val = bincode::serialize(val).unwrap();
        let existing = BS::insert(&self, &key, &val).unwrap();
        Ok(match existing {
            Some(bytes) => bincode::deserialize(&bytes).unwrap(),
            None => None,
        })
    }

    fn atomic_update(&self, key: &K, mut op: impl FnMut(V) -> V + Clone) -> Result<(), Self::Error> {
        let key = bincode::serialize(key).unwrap();
        let bytes_op = |old: &[u8]| -> Vec<u8> {
            let old: V = bincode::deserialize(old).unwrap(); // TODO error handling
            let new = op(old);
            bincode::serialize(&new).unwrap()
        };
        BS::atomic_update(&self, &key, bytes_op)
    }

    fn conditional_update(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error> {
        let key = bincode::serialize(key).unwrap();
        let new = bincode::serialize(new).unwrap();
        let expected = bincode::serialize(expected).unwrap();
        BS::conditional_update(&self, &key, &new, &expected).unwrap();
        Ok(())
    }
}
