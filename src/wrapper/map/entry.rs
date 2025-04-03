use std::fmt;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{DataStore, Error};

use super::Map;

pub enum Entry<'a, Key, Value, DS>
where
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore,
{
    Occupied(OccupiedEntry<'a, Key, Value, DS>),
    Vacent(VacentEntry<'a, Key, Value, DS>),
}

impl<'a, Key, Value, E, DS> Entry<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u16>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().entry("poneyland").or_insert(3);
    /// assert_eq!(map.get()?, Some(3));
    ///
    /// db.map().entry("poneyland").or_insert(10);
    /// assert_eq!(map["poneyland"], 3);
    /// # Ok(())
    /// # }
    /// use std::collections::HashMap;
    ///
    /// let mut map: HashMap<&str, u32> = HashMap::new();
    ///
    /// map.entry("poneyland").or_insert(3);
    /// assert_eq!(map["poneyland"], 3);
    ///
    /// *map.entry("poneyland").or_insert(10) *= 2;
    /// assert_eq!(map["poneyland"], 6);
    /// ```
    pub fn or_insert(self, default: Value) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied_entry) => occupied_entry.get(),
            Entry::Vacent(vacent_entry) => {
                let key = vacent_entry.key();
                vacent_entry.map.insert(&key, &default)?;
                Ok(default)
            }
        }
    }
    pub fn or_insert_with<F: FnOnce() -> Value>(self, default: F) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied_entry) => occupied_entry.get(),
            Entry::Vacent(vacent_entry) => {
                let key = vacent_entry.key();
                let value = default();
                vacent_entry.map.insert(&key, &value)?;
                Ok(value)
            }
        }
    }
    pub fn or_insert_with_key<F: FnOnce(&Key) -> Value>(
        self,
        default: F,
    ) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied_entry) => occupied_entry.get(),
            Entry::Vacent(vacent_entry) => {
                let key = vacent_entry.key();
                let value = default(key);
                vacent_entry.map.insert(&key, &value)?;
                Ok(value)
            }
        }
    }
    pub fn key(&self) -> &Key {
        match self {
            Entry::Occupied(occupied) => occupied.key(),
            Entry::Vacent(vacent) => vacent.key(),
        }
    }
    pub fn and_modify<F>(self, f: F) -> Result<Self, Error<E>>
    where
        F: FnOnce(&mut Value),
    {
        match self {
            Entry::Occupied(ref occupied) => {
                let mut value = occupied.get()?;
                f(&mut value);
                occupied.insert(value)?;
            }
            Entry::Vacent(_) => (),
        }
        Ok(self)
    }
    pub fn insert_entry(self, value: Value) -> Result<OccupiedEntry<'a, Key, Value, DS>, Error<E>> {
        match self {
            Entry::Occupied(occupied) => {
                occupied.insert(value)?;
                Ok(occupied)
            }
            Entry::Vacent(vacent) => vacent.insert_entry(value),
        }
    }
}

impl<'a, Key, Value, E, DS> Entry<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned + Default,
    DS: DataStore<DbError = E>,
{
    pub fn or_default(self) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied) => occupied.get(),
            Entry::Vacent(_) => Ok(Value::default()),
        }
    }
}

pub struct OccupiedEntry<'a, Key, Value, DS>
where
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore,
{
    key: Key,
    map: &'a Map<Key, Value, DS>,
}

impl<'a, Key, Value, E, DS> OccupiedEntry<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    pub fn get(&self) -> Result<Value, Error<E>> {
        Ok(self.map.get(&self.key)?.expect("occupied entry"))
    }
    pub fn key(&self) -> &Key {
        &self.key
    }
    /// Sets the value of the entry, and returns the entry’s old value.
    pub fn insert(&self, value: Value) -> Result<Value, Error<E>> {
        Ok(self.map.insert(&self.key, &value)?.expect("occupied entry"))
    }
    /// Takes the value out of the entry, and returns it.
    pub fn remove(&self) -> Result<Value, Error<E>> {
        Ok(self.map.remove(&self.key)?.expect("occupied_entry"))
    }
    pub fn remove_entry(self) -> Result<(Key, Value), Error<E>> {
        let val = self.map.remove(&self.key)?.expect("occupied entry");
        Ok((self.key, val))
    }
}

pub struct VacentEntry<'a, Key, Value, DS>
where
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore,
{
    pub(crate) key: Key,
    pub(crate) map: &'a Map<Key, Value, DS>,
}

impl<'a, Key, Value, E, DS> VacentEntry<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: crate::DataStore<DbError = E>,
{
    pub fn insert(self, value: Value) -> Result<(), Error<E>> {
        self.map.insert(&self.key, &value)?;
        Ok(())
    }
    pub fn insert_entry(self, value: Value) -> Result<OccupiedEntry<'a, Key, Value, DS>, Error<E>> {
        self.map.insert(&self.key, &value)?;
        Ok(OccupiedEntry {
            key: self.key,
            map: self.map,
        })
    }
    pub fn into_key(self) -> Key {
        self.key
    }
    pub fn key(&self) -> &Key {
        &self.key
    }
}

impl<Key, Value, E, DS> Map<Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// let mut letters = HashMap::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     letters.entry(ch).and_modify(|counter| *counter += 1).or_insert(1);
    /// }
    ///
    /// assert_eq!(letters[&'s'], 2);
    /// assert_eq!(letters[&'t'], 3);
    /// assert_eq!(letters[&'u'], 1);
    /// assert_eq!(letters.get(&'y'), None);
    /// ```
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
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
    ///	    letters: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// for ch in "a short treatise on fungi".chars() {
    ///     db.letters().entry(ch)?.and_modify(|counter| *counter += 1)?.or_insert(1)?;
    /// }
    ///
    /// assert_eq!(db.letters()[&'s'], 2);
    /// assert_eq!(db.letters()[&'t'], 3);
    /// assert_eq!(db.letters()[&'u'], 1);
    /// assert_eq!(db.letters.get(&'y'), None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn entry(&self, key: Key) -> Result<Entry<'_, Key, Value, DS>, Error<E>> {
        if self.contains_key(&key)? {
            Ok(Entry::Occupied(OccupiedEntry { key, map: &self }))
        } else {
            Ok(Entry::Vacent(VacentEntry { map: &self, key }))
        }
    }
}
