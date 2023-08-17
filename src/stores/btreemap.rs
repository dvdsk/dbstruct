use std::collections;
use std::sync::{Arc, RwLock};

use crate::traits::ByteStore;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("rwlock was poisoned (another thread panicked while holding the lock)")]
    Poisoned,
}

/// This is a very simple backend that offers no persistance but also needs no path argument. It only
/// supports some wrapper. Use it for testing.
///
/// ### ALL CHANGES ARE LOST WHEN THE OBJECT IS DROPPED
/// again: use for testing the api only
///
#[derive(Default, Clone)]
pub struct BTreeMap(Arc<RwLock<collections::BTreeMap<Vec<u8>, Vec<u8>>>>);

impl BTreeMap {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ByteStore for BTreeMap {
    type Error = Error;
    type Bytes = Vec<u8>;

    fn get(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        let map = self.0.read().map_err(|_| Self::Error::Poisoned)?;
        Ok(map.get(key).cloned())
    }

    fn remove(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        let mut map = self.0.write().map_err(|_| Self::Error::Poisoned)?;
        Ok(map.remove(key))
    }

    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        let mut map = self.0.write().map_err(|_| Self::Error::Poisoned)?;
        Ok(map.insert(key.to_vec(), val.to_vec()))
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

impl crate::traits::byte_store::Ordered for BTreeMap {
    fn get_lt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::Error> {
        let map = self.0.write().map_err(|_| Self::Error::Poisoned)?;
        let zero = vec![0];
        let range = zero..=key.to_vec();
        let Some((k,v)) = map.range(range).next_back() else {
            return Ok(None);
        };
        Ok(Some((k.to_vec(), v.to_vec())))
    }
    fn get_gt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::Error> {
        use std::ops::Bound::*;
        let map = self.0.write().map_err(|_| Self::Error::Poisoned)?;
        let range = (Excluded(key.to_vec()), Unbounded);
        let Some((k,v)) = map.range(range).next() else {
            return Ok(None);
        };
        Ok(Some((k.to_vec(), v.to_vec())))
    }
}

#[cfg(test)]
mod tests {
    use super::BTreeMap;
    use crate::traits::data_store::Ordered;
    use crate::traits::DataStore;

    #[test]
    fn get_then_insert() {
        let ds = BTreeMap::new();
        let existing = ds.insert(&1, &2).unwrap();
        assert_eq!(existing, None);
        let val: u8 = ds.remove(&1).unwrap().unwrap();
        assert_eq!(val, 2);
    }

    #[test]
    fn get_lt() {
        let ds = BTreeMap::new();
        ds.insert(&1, &2).unwrap();
        ds.insert(&10, &4).unwrap();
        ds.insert(&20, &8).unwrap();
        let (key, val): (u8, u8) = ds.get_lt(&11).unwrap().unwrap();
        assert_eq!(key, 10);
        assert_eq!(val, 4);
    }

    #[test]
    fn get_gt() {
        let ds = BTreeMap::new();
        ds.insert(&1, &2).unwrap();
        ds.insert(&10, &4).unwrap();
        ds.insert(&20, &8).unwrap();
        let (key, val): (u8, u8) = ds.get_gt(&10).unwrap().unwrap();
        assert_eq!(key, 20);
        assert_eq!(val, 8);
    }
}
