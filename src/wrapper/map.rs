use core::fmt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use tracing::{instrument, trace};

use crate::traits::{byte_store, DataStore};
use crate::Error;

mod extend;
mod iterator;

/// mimics the API of [`HashMap`][std::collections::HashMap]
pub struct Map<'a, Key, Value, DS>
where
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore,
{
    phantom_key: PhantomData<&'a Key>,
    phantom_val: PhantomData<Value>,
    tree: DS,
    prefix: u8,
}

#[derive(Serialize)]
pub struct Prefixed<'a, K> {
    prefix: u8,
    key: &'a K,
}

impl<'a, Key, Value, E, DS> Map<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    #[doc(hidden)]
    #[instrument(skip(tree), level = "debug")]
    pub fn new(tree: DS, prefix: u8) -> Self {
        Self {
            phantom_key: PhantomData,
            phantom_val: PhantomData,
            tree,
            prefix,
        }
    }

    fn prefix(&self, key: &'a Key) -> Prefixed<'a, Key> {
        trace!("prefixing key with: {}", self.prefix);
        Prefixed {
            prefix: self.prefix,
            key,
        }
    }

    /// returns existing value if any was set
    #[instrument(skip_all, level = "debug")]
    pub fn insert(&self, key: &'a Key, value: &'a Value) -> Result<Option<Value>, Error<E>> {
        let key = self.prefix(key);
        let existing = self.tree.insert(&key, value)?;
        Ok(existing)
    }

    #[instrument(skip_all, level = "debug")]
    pub fn get(&self, key: &'a Key) -> Result<Option<Value>, Error<E>> {
        let key = self.prefix(key);
        let value = self.tree.get(&key)?;
        Ok(value)
    }

    #[instrument(skip_all, level = "debug")]
    pub fn remove(&self, key: &'a Key) -> Result<Option<Value>, Error<E>> {
        let key = self.prefix(key);
        let value = self.tree.remove(&key)?;
        Ok(value)
    }
}

impl<Key, Value, E, DS> Map<'_, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E> + byte_store::Ordered<DbError = E>,
{
    /// Clears the map, removing all key-value pairs.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::DataStore;
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// pub struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    /// let db = Test::new().unwrap();
    /// db.map().insert(&1, &"a".to_owned());
    /// db.map().clear();
    /// assert!(db.map().is_empty());
    /// ```
    pub fn clear(&self) -> Result<(), Error<E>> {
        for key in self.keys() {
            let key = key?;
            self.remove(&key)?;
        }

        Ok(())
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::DataStore;
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// pub struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    /// let db = Test::new().unwrap();
    /// db.map().insert(&1, &"a".to_owned());
    /// db.map().clear();
    /// assert!(db.map().is_empty());
    /// ```
    pub fn is_empty(&self) -> Result<bool, Error<E>> {
        Ok(self.iter().next().is_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores;

    pub(crate) type TestMap<'a, K, V> = Map<'a, K, V, stores::BTreeMap>;
    pub(crate) fn empty<'a, K, V>() -> TestMap<'a, K, V>
    where
        K: Clone + Serialize + DeserializeOwned,
        V: Clone + Serialize + DeserializeOwned,
    {
        let ds = stores::BTreeMap::new();
        let map = Map::new(ds, 1);
        map
    }
}
