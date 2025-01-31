use core::fmt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Borrow;
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
pub struct Prefixed<'a, K: ?Sized> {
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

    fn prefix<Q>(&self, key: &'a Q) -> Prefixed<'a, Q>
    where
        Key: Borrow<Q>,
        Q: Serialize + ?Sized,
    {
        trace!("prefixing key with: {}", self.prefix);
        Prefixed {
            prefix: self.prefix,
            key,
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert_eq!(db.map().insert(&37, &"a".to_owned())?, None);
    /// assert_eq!(db.map().is_empty()?, false);
    ///
    /// db.map().insert(&37, &"b".to_owned())?;
    /// assert_eq!(db.map().insert(&37, &"c".to_owned())?, Some("b".to_owned()));
    /// assert_eq!(db.map().get(&37)?, Some("c".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all, level = "debug")]
    pub fn insert<K>(&self, key: &'a K, value: &'a Value) -> Result<Option<Value>, Error<E>>
    where
        Key: std::borrow::Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = self.prefix(key);
        let existing = self.tree.insert(&key, value)?;
        Ok(existing)
    }

    /// Returns a copy of the value corresponding to the key.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().insert(&1, &"a".to_owned())?;
    /// assert_eq!(db.map().get(&1)?, Some("a".to_owned()));
    /// assert_eq!(db.map().get(&2)?, None);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all, level = "debug")]
    pub fn get<K>(&self, key: &'a K) -> Result<Option<Value>, Error<E>>
    where
        Key: std::borrow::Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = self.prefix(key);
        let value = self.tree.get(&key)?;
        Ok(value)
    }

    /// Returns a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().insert(&1, &"a".to_owned())?;
    /// assert_eq!(db.map().remove(&1)?, Some("a".to_owned()));
    /// assert_eq!(db.map().remove(&2)?, None);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all, level = "debug")]
    pub fn remove<K>(&self, key: &'a K) -> Result<Option<Value>, Error<E>>
    where
        Key: std::borrow::Borrow<K>,
        K: Serialize + ?Sized,
    {
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
    DS: DataStore<DbError = E> + byte_store::Ordered,
    Error<E>: From<Error<<DS as crate::ByteStore>::DbError>>,
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
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().insert(&1, &"a".to_owned())?;
    /// db.map().clear()?;
    /// assert!(db.map().is_empty()?);
    /// # Ok(())
    /// # }
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
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert!(db.map().is_empty()?);
    /// db.map().insert(&1, &"a".to_owned())?;
    /// assert!(!db.map().is_empty()?);
    /// # Ok(())
    /// # }
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
