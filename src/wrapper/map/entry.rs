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
    Vacant(VacantEntry<'a, Key, Value, DS>),
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
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
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
    /// db.map().entry("poneyland".to_string())?.or_insert(3)?;
    /// assert_eq!(db.map().get("poneyland")?, Some(3));
    ///
    /// db.map().entry("poneyland".to_string())?.or_insert(10)?;
    /// assert_eq!(db.map().get("poneyland")?, Some(3));
    /// # Ok(())
    /// # }
    /// ```
    pub fn or_insert(self, default: Value) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied_entry) => occupied_entry.get(),
            Entry::Vacant(vacent_entry) => {
                let key = vacent_entry.key();
                vacent_entry.map.insert(&key, &default)?;
                Ok(default)
            }
        }
    }
    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
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
    ///	    map: HashMap<String, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    ///
    /// let value = "hoho".to_string();
    /// db.map().entry("poneyland".to_string())?.or_insert_with(|| value)?;
    ///
    /// assert_eq!(db.map().get("poneyland")?, Some("hoho".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn or_insert_with<F: FnOnce() -> Value>(self, default: F) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied_entry) => occupied_entry.get(),
            Entry::Vacant(vacent_entry) => {
                let key = vacent_entry.key();
                let value = default();
                vacent_entry.map.insert(&key, &value)?;
                Ok(value)
            }
        }
    }
    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, usize>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().entry("poneyland".to_string())?
    ///     .or_insert_with_key(|key| key.chars().count())?;
    ///
    /// assert_eq!(db.map().get("poneyland")?, Some(9));
    /// # Ok(())
    /// # }
    /// ```
    pub fn or_insert_with_key<F: FnOnce(&Key) -> Value>(
        self,
        default: F,
    ) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied_entry) => occupied_entry.get(),
            Entry::Vacant(vacent_entry) => {
                let key = vacent_entry.key();
                let value = default(key);
                vacent_entry.map.insert(&key, &value)?;
                Ok(value)
            }
        }
    }
    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert_eq!(db.map().entry("poneyland".to_string())?.key(), &"poneyland".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn key(&self) -> &Key {
        match self {
            Entry::Occupied(occupied) => occupied.key(),
            Entry::Vacant(vacent) => vacent.key(),
        }
    }
    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    ///
    /// db.map().entry("poneyland".to_string())?
    ///    .and_modify(|e| { *e += 1 })?
    ///    .or_insert(42)?;
    /// assert_eq!(db.map().get("poneyland")?, Some(42));
    ///
    /// db.map().entry("poneyland".to_string())?
    ///    .and_modify(|e| { *e += 1 })?
    ///    .or_insert(42)?;
    /// assert_eq!(db.map().get("poneyland")?, Some(43));
    /// # Ok(())
    /// # }
    /// ```
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
            Entry::Vacant(_) => (),
        }
        Ok(self)
    }

    /// Sets the value of the entry, and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// let map = db.map();
    /// let entry = map.entry("poneyland".to_string())?
    ///     .insert_entry("hoho".to_string())?;
    ///
    /// assert_eq!(entry.key(), &"poneyland".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn insert_entry(self, value: Value) -> Result<OccupiedEntry<'a, Key, Value, DS>, Error<E>> {
        match self {
            Entry::Occupied(occupied) => {
                occupied.insert(value)?;
                Ok(occupied)
            }
            Entry::Vacant(vacent) => vacent.insert_entry(value),
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
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().entry("poneyland".to_string())?.or_default()?;
    ///
    /// assert_eq!(db.map().get("poneyland")?, Some(0));
    /// # Ok(())
    /// # }
    /// ```
    pub fn or_default(self) -> Result<Value, Error<E>> {
        match self {
            Entry::Occupied(occupied) => occupied.get(),
            Entry::Vacant(vacent) => {
                vacent.insert(Value::default())?;
                Ok(Value::default())
            }
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
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().entry("poneyland".to_string())?.or_insert(12)?;
    /// assert_eq!(db.map().entry("poneyland".to_string())?.key(), &"poneyland".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::wrapper::map::Entry;
    ///
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    ///
    /// db.map().entry("poneyland".to_string())?.or_insert(12)?;
    ///
    /// if let Entry::Occupied(o) = db.map().entry("poneyland".to_string())? {
    ///     assert_eq!(o.get()?, 12);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self) -> Result<Value, Error<E>> {
        Ok(self.map.get(&self.key)?.expect("occupied entry"))
    }
    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::wrapper::map::Entry;
    ///
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    ///
    /// db.map().entry("poneyland".to_string())?.or_insert(12)?;
    ///
    /// if let Entry::Occupied(mut o) = db.map().entry("poneyland".to_string())? {
    ///     assert_eq!(o.insert(15)?, 12);
    /// }
    ///
    /// assert_eq!(db.map().get("poneyland")?, Some(15));
    /// # Ok(())
    /// # }
    /// ```
    /// Sets the value of the entry, and returns the entry’s old value.
    pub fn insert(&self, value: Value) -> Result<Value, Error<E>> {
        Ok(self.map.insert(&self.key, &value)?.expect("occupied entry"))
    }
    /// Takes the value out of the entry, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::wrapper::map::Entry;
    ///
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().entry("poneyland".to_string())?.or_insert(12)?;
    ///
    /// if let Entry::Occupied(o) = db.map().entry("poneyland".to_string())? {
    ///     assert_eq!(o.remove()?, 12);
    /// }
    ///
    /// assert_eq!(db.map().contains_key("poneyland")?, false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove(&self) -> Result<Value, Error<E>> {
        Ok(self.map.remove(&self.key)?.expect("occupied_entry"))
    }

    /// Take the ownership of the key and value from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::wrapper::map::Entry;
    ///
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().entry("poneyland".to_string())?.or_insert(12)?;
    ///
    /// if let Entry::Occupied(o) = db.map().entry("poneyland".to_string())? {
    ///     // We delete the entry from the map.
    ///     o.remove_entry()?;
    /// }
    ///
    /// assert_eq!(db.map().contains_key("poneyland")?, false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_entry(self) -> Result<(Key, Value), Error<E>> {
        let val = self.map.remove(&self.key)?.expect("occupied entry");
        Ok((self.key, val))
    }
}

pub struct VacantEntry<'a, Key, Value, DS>
where
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore,
{
    pub(crate) key: Key,
    pub(crate) map: &'a Map<Key, Value, DS>,
}

impl<'a, Key, Value, E, DS> VacantEntry<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: crate::DataStore<DbError = E>,
{
    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::wrapper::map::Entry;
    ///
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    ///
    /// if let Entry::Vacant(o) = db.map().entry("poneyland".to_string())? {
    ///     o.insert(37)?;
    /// }
    /// assert_eq!(db.map().get("poneyland")?, Some(37));
    /// # Ok(())
    /// # }
    /// ```
    pub fn insert(self, value: Value) -> Result<(), Error<E>> {
        self.map.insert(&self.key, &value)?;
        Ok(())
    }
    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::wrapper::map::Entry;
    ///
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    ///
    /// if let Entry::Vacant(o) = db.map().entry("poneyland".to_string())? {
    ///     o.insert_entry(37)?;
    /// }
    /// assert_eq!(db.map().get("poneyland")?, Some(37));
    /// # Ok(())
    /// # }
    /// ```
    pub fn insert_entry(self, value: Value) -> Result<OccupiedEntry<'a, Key, Value, DS>, Error<E>> {
        self.map.insert(&self.key, &value)?;
        Ok(OccupiedEntry {
            key: self.key,
            map: self.map,
        })
    }
    /// Take ownership of the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbstruct::wrapper::map::Entry;
    ///
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    ///
    /// if let Entry::Vacant(v) = db.map().entry("poneyland".to_string())? {
    ///     v.into_key();
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_key(self) -> Key {
        self.key
    }
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<String, u32>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert_eq!(
    ///     db.map().entry("poneyland".to_string())?.key(), 
    ///     &"poneyland".to_string()
    /// );
    /// # Ok(())
    /// # }
    /// ```
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
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    letters: HashMap<char, u16>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// for ch in "a short treatise on fungi".chars() {
    ///     db.letters().entry(ch)?.and_modify(|counter| *counter += 1)?.or_insert(1)?;
    /// }
    ///
    /// assert_eq!(db.letters().get(&'s')?, Some(2));
    /// assert_eq!(db.letters().get(&'t')?, Some(3));
    /// assert_eq!(db.letters().get(&'u')?, Some(1));
    /// assert_eq!(db.letters().get(&'y')?, None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn entry(&self, key: Key) -> Result<Entry<'_, Key, Value, DS>, Error<E>> {
        if self.contains_key(&key)? {
            Ok(Entry::Occupied(OccupiedEntry { key, map: &self }))
        } else {
            Ok(Entry::Vacant(VacantEntry { map: &self, key }))
        }
    }
}
