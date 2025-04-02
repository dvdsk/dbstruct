use std::collections;
use std::sync::{Arc, RwLock};

use crate::traits::{byte_store, ByteStore};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("rwlock was poisoned (another thread panicked while holding the lock)")]
    Poisoned,
}

/// This is a very simple backend that offers no persistence but also needs no
/// path argument. It only supports some wrapper. Use it for testing.
///
/// ### ALL CHANGES ARE LOST WHEN THE OBJECT IS DROPPED
/// again: use for testing the API only
///
#[derive(Default, Clone)]
pub struct BTreeMap(Arc<RwLock<collections::BTreeMap<Vec<u8>, Vec<u8>>>>);

impl BTreeMap {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ByteStore for BTreeMap {
    type DbError = Error;
    type Bytes = Vec<u8>;

    fn get(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError> {
        let map = self.0.read().map_err(|_| Self::DbError::Poisoned)?;
        Ok(map.get(key).cloned())
    }

    fn remove(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError> {
        let mut map = self.0.write().map_err(|_| Self::DbError::Poisoned)?;
        Ok(map.remove(key))
    }

    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError> {
        let mut map = self.0.write().map_err(|_| Self::DbError::Poisoned)?;
        Ok(map.insert(key.to_vec(), val.to_vec()))
    }
}

impl byte_store::Ordered for BTreeMap {
    fn get_lt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::DbError> {
        let map = self.0.write().map_err(|_| Self::DbError::Poisoned)?;
        let zero = vec![0];
        let range = zero..=key.to_vec();
        let Some((k, v)) = map.range(range).next_back() else {
            return Ok(None);
        };
        dbg!(k);
        Ok(Some((k.to_vec(), v.to_vec())))
    }
    fn get_gt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::DbError> {
        use std::ops::Bound::*;
        let map = self.0.write().map_err(|_| Self::DbError::Poisoned)?;
        let range = (Excluded(key.to_vec()), Unbounded);
        let Some((k, v)) = map.range(range).next() else {
            return Ok(None);
        };
        Ok(Some((k.to_vec(), v.to_vec())))
    }
}

impl byte_store::Atomic for BTreeMap {
    fn atomic_update(
        &self,
        key: &[u8],
        mut op: impl FnMut(Option<&[u8]>) -> Option<Vec<u8>>,
    ) -> Result<(), Self::DbError> {
        let mut map = self.0.write().map_err(|_| Self::DbError::Poisoned)?;
        let curr = map.get(key).map(|c| c.as_slice());
        if let Some(new) = op(curr) {
            map.insert(key.to_vec(), new);
        }
        Ok(())
    }

    fn conditional_update(
        &self,
        key: &[u8],
        new: &[u8],
        expected: &[u8],
    ) -> Result<(), Self::DbError> {
        let mut map = self.0.write().map_err(|_| Self::DbError::Poisoned)?;
        let curr = map.get(key).map(|c| c.as_slice());
        if let Some(curr) = curr {
            if curr == expected {
                map.insert(key.to_vec(), new.to_vec());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
impl BTreeMap {
    pub(crate) fn force_error(&self) {
        // poison the lock such that we get an error on the next use of self
        let map = self.0.clone();
        let handle = std::thread::spawn(move || {
            let _lock = map.write().unwrap();
            panic!("panicking here to poinson the lock")
        });

        let res = handle.join();
        assert!(res.is_err());
    }
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::EnvFilter;

    use super::BTreeMap;
    use crate::traits::data_store::Ordered;
    use crate::traits::DataStore;

    #[test]
    fn get_then_insert() {
        let ds = BTreeMap::new();
        let existing: Option<u16> = ds.insert(&1, &2).unwrap();
        assert_eq!(existing, None);
        let val: u16 = ds.remove(&1).unwrap().unwrap();
        assert_eq!(val, 2);
    }

    #[test]
    fn get_lt() {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        let ds = BTreeMap::new();
        ds.insert::<u16, u16, u16>(&1, &2).unwrap();
        ds.insert::<u16, u16, u16>(&10, &4).unwrap();
        ds.insert::<u16, u16, u16>(&20, &8).unwrap();

        let (key, val): (u16, u16) = ds.get_lt::<u16, u16, u16>(&11).unwrap().unwrap();
        assert_eq!(key, 10);
        assert_eq!(val, 4);
    }

    #[test]
    fn get_gt() {
        let ds = BTreeMap::new();
        ds.insert::<u16, u16, u16>(&1, &2).unwrap();
        ds.insert::<u16, u16, u16>(&10, &4).unwrap();
        ds.insert::<u16, u16, u16>(&20, &8).unwrap();
        let (key, val): (u16, u16) = ds.get_gt::<u16, u16, u16>(&10).unwrap().unwrap();
        assert_eq!(key, 20);
        assert_eq!(val, 8);
    }
}
