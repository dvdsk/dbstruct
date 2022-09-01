use std::collections;
use std::sync::{Arc, RwLock};

use crate::traits::ByteStore;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("rwlock was poisoned (another thread panicked while holding the lock)")]
    Poisoned,
}

/// This is a very simple backend that offers no persistance but also needs no path argument. It only
/// supports some wrappers. Use it for testing.
///
/// ### ALL CHANGES ARE LOST WHEN THE OBJECT IS DROPPED
/// again: use for testing the api only
///
#[derive(Default, Clone)]
pub struct HashMap(Arc<RwLock<collections::HashMap<Vec<u8>, Vec<u8>>>>);

impl HashMap {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ByteStore for HashMap {
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
mod tests {
    use super::HashMap;
    use crate::traits::DataStore;

    #[test]
    fn get_then_insert() {
        let ds = HashMap::new();
        let existing = ds.insert(&1, &2).unwrap();
        assert_eq!(existing, None);
        let val: u8 = ds.remove(&1).unwrap().unwrap();
        assert_eq!(val, 2);
    }
}
