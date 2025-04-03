use core::fmt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use tracing::instrument;

use crate::traits::{byte_store, DataStore};
use crate::Error;

use super::PhantomUnsync;

mod entry;
mod extend;
mod iterator;

/// mimics the API of [`HashMap`][std::collections::HashMap]
pub struct Map<Key, Value, DS>
where
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore,
{
    phantom_key: PhantomData<Key>,
    phantom_val: PhantomData<Value>,
    phantom2: PhantomUnsync,
    tree: DS,
    prefix: u8,
}

#[derive(Serialize)]
pub struct Prefixed<'a, K: ?Sized> {
    prefix: u8,
    key: &'a K,
}

impl<Key, Value, E, DS> Map<Key, Value, DS>
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
            phantom2: PhantomData,
            tree,
            prefix,
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though.
    ///
    /// The key and or value may be any borrowed form of the map’s key and or
    /// value type, but the serialized form must match those for the key and
    /// or value type.
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
    /// assert_eq!(db.map().insert(&37, "a")?, None);
    /// assert_eq!(db.map().is_empty()?, false);
    ///
    /// db.map().insert(&37, "b")?;
    /// assert_eq!(db.map().insert(&37, "c")?, Some("b".to_owned()));
    /// assert_eq!(db.map().get(&37)?, Some("c".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all, level = "debug")]
    pub fn insert<K, V>(&self, key: &K, value: &V) -> Result<Option<Value>, Error<E>>
    where
        Key: std::borrow::Borrow<K>,
        K: Serialize + ?Sized,
        Value: std::borrow::Borrow<V>,
        V: Serialize + ?Sized,
    {
        let key = Prefixed {
            prefix: self.prefix,
            key,
        };
        let existing = self.tree.insert(&key, value)?;
        Ok(existing)
    }

    /// Returns a copy of the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map’s key type, but the
    /// serialized form must match that of the owned key type.
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
    /// db.map().insert(&1, "a")?;
    /// assert_eq!(db.map().get(&1)?, Some("a".to_owned()));
    /// assert_eq!(db.map().get(&2)?, None);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all, level = "debug")]
    pub fn get<K>(&self, key: &K) -> Result<Option<Value>, Error<E>>
    where
        Key: std::borrow::Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = Prefixed {
            prefix: self.prefix,
            key,
        };
        let value = self.tree.get(&key)?;
        Ok(value)
    }

    /// Returns a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map’s key type, but the
    /// serialized form must match that of the owned key type.
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
    /// db.map().insert(&1, "a")?;
    /// assert_eq!(db.map().remove(&1)?, Some("a".to_owned()));
    /// assert_eq!(db.map().remove(&2)?, None);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all, level = "debug")]
    pub fn remove<K>(&self, key: &K) -> Result<Option<Value>, Error<E>>
    where
        Key: std::borrow::Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = Prefixed {
            prefix: self.prefix,
            key,
        };
        let value = self.tree.remove(&key)?;
        Ok(value)
    }

    /// Returns `true` if the map contains a value for the specific key.
    ///
    /// The key may be any borrowed form of the map’s key type, but the
    /// serialized form must match that of the owned key type.
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
    /// db.map().insert(&1, "a")?;
    /// assert_eq!(db.map().contains_key(&1)?, true);
    /// assert_eq!(db.map().contains_key(&2)?, false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains_key<K>(&self, key: &K) -> Result<bool, Error<E>>
    where
        Key: std::borrow::Borrow<K>,
        K: Serialize + ?Sized,
    {
        let key = Prefixed {
            prefix: self.prefix,
            key,
        };
        let value: Option<Value> = self.tree.get(&key)?;
        Ok(value.is_some())
    }
}

impl<Key, Value, E, DS> Map<Key, Value, DS>
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

impl<Key, Value, E, DS> fmt::Debug for Map<Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned + fmt::Debug,
    Value: Serialize + DeserializeOwned + fmt::Debug,
    DS: DataStore<DbError = E> + byte_store::Ordered,
    Error<E>: From<Error<<DS as crate::ByteStore>::DbError>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[\n")?;
        for element in self.iter() {
            match element {
                Ok((key, val)) => f.write_fmt(format_args!("    {key:?}: {val:?},\n"))?,
                Err(err) => {
                    f.write_fmt(format_args!(
                        "ERROR while printing full list, could \
                         not read next element from db: {err}"
                    ))?;
                    return Ok(());
                }
            }
        }
        f.write_str("]\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores;

    pub(crate) type TestMap<'a, K, V> = Map<K, V, stores::BTreeMap>;
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
