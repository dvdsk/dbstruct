use core::fmt;
use serde::{de::DeserializeOwned, Serialize};

pub trait DataStore<T> {
    type Error: fmt::Debug;
    fn get(&self, key: &T) -> Result<T, Self::Error>;
    fn insert(&self, key: T, val: T) -> Result<(), Self::Error>;
    fn atomic_update(
        &self,
        key: T,
        op: impl FnMut(T) -> T + Clone,
    ) -> Result<(), Self::Error>;
    fn conditional_update(&self, key: T, new: T, expected: T) -> Result<(), Self::Error>;
}

trait BytesStore<T> {
    type Error: fmt::Debug;
    fn get(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>;
    fn insert(&self, key: &[u8], val: &[u8]) -> Result<(), Self::Error>;
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

impl<T, E, BS> DataStore<T> for BS 
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    BS: BytesStore<T, Error = E>,
{
    type Error = E;

    fn get(&self, key: &T) -> Result<T, Self::Error> {
        let key = bincode::serialize(key).unwrap();
        let val = BS::get(&self, &key).unwrap();
        let val = bincode::deserialize(&val).unwrap();
        Ok(val)
    }

    fn insert(&self, key: T, val: T) -> Result<(), Self::Error> {
        let key = bincode::serialize(&key).unwrap();
        let val = bincode::serialize(&val).unwrap();
        BS::insert(&self, &key, &val).unwrap();
        Ok(())
    }

    fn atomic_update(
        &self,
        key: T,
        mut op: impl FnMut(T) -> T + Clone,
    ) -> Result<(), Self::Error> {
        let key = bincode::serialize(&key).unwrap();
        let bytes_op = |old: &[u8]| -> Vec<u8> {
            let old: T =  bincode::deserialize(old).unwrap(); // TODO error handling
            let new = op(old);
            bincode::serialize(&new).unwrap()
        };
        BS::atomic_update(&self, &key, bytes_op)
    }

    fn conditional_update(&self, key: T, new: T, expected: T) -> Result<(), Self::Error> {
        let key = bincode::serialize(&key).unwrap();
        let new = bincode::serialize(&new).unwrap();
        let expected = bincode::serialize(&expected).unwrap();
        BS::conditional_update(&self, &key, &new, &expected).unwrap();
        Ok(())
    }
}
