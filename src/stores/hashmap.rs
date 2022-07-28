use std::collections::HashMap;
use std::hash;
use std::sync::RwLock;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::DataStore;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("rwlock was poisoned (another thread panicked while holding the lock)")]
    Poisoned,
}

impl<K, V> DataStore<K, V> for RwLock<HashMap<K, V>>
where
    K: Serialize + Eq + hash::Hash + Clone,
    V: Serialize + DeserializeOwned + Clone,
{
    type Error = Error;

    fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let map = self.read().map_err(|_| Self::Error::Poisoned)?;
        Ok(map.get(key).cloned())
    }

    fn remove(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let mut map = self.write().map_err(|_| Self::Error::Poisoned)?;
        Ok(map.remove(key))
    }

    fn insert<'a>(&self, key: &'a K, val: &'a V) -> Result<Option<V>, Self::Error> {
        let mut map = self.write().map_err(|_| Self::Error::Poisoned)?;
        Ok(map.insert(key.clone(), val.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_then_insert() {
        let ds = RwLock::new(HashMap::new());
        let existing = ds.insert(&1, &2).unwrap();
        assert_eq!(existing, None);
        let val = ds.remove(&1).unwrap().unwrap();
        assert_eq!(val, 2);
    }
}
