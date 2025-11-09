use core::fmt;
use std::borrow::Borrow;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::{data_store, DataStore};
use crate::Error;

/// Here missing values are represented by [`Option::None`].
pub struct OptionValue<T, DS>
where
    DS: DataStore,
{
    phantom: PhantomData<T>,
    ds: DS,
    key: u8,
}

impl<T, E, DS> OptionValue<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    #[doc(hidden)]
    pub fn new(ds: DS, key: u8) -> Self {
        Self {
            phantom: PhantomData,
            ds,
            key,
        }
    }

    /// Sets the value of this database item.
    ///
    /// The argument may be any borrowed form of value type, but the
    /// serialized form must match that of the value type.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///     name: Option<String>,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.name().set("Artemis")?;
    /// assert_eq!(db.name().get()?, Some("Artemis".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set<Q>(&mut self, value: Option<&Q>) -> Result<(), Error<E>>
    where
        T: Borrow<Q>,
        Q: Serialize + ?Sized,
    {
        if let Some(v) = value {
            self.ds.insert::<_, Q, T>(&self.key, v)?;
        } else {
            self.ds.clear::<_>(&self.key)?;
        }
        Ok(())
    }

    /// Get the current value of this item.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///     name: Option<String>,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert_eq!(db.name().get()?, None);
    /// db.name().set("Artemis")?;
    /// assert_eq!(db.name().get()?, Some("Artemis".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self) -> Result<Option<T>, Error<E>> {
        self.ds.get(&self.key)
    }

    /// Returns `true` if the option is a `None` value.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///     name: Option<String>,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert!(db.name().is_none()?);
    /// db.name().set("Artemis")?;
    /// assert!(!db.name().is_none()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_none(&self) -> Result<bool, Error<E>> {
        self.ds.contains(&self.key)
    }

    /// Returns `true` if the option is a `Some` value.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///     name: Option<String>,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert!(!db.name().is_some()?);
    /// db.name().set("Artemis")?;
    /// assert!(db.name().is_some()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_some(&self) -> Result<bool, Error<E>> {
        self.is_none().map(|b| !b)
    }
}

impl<T, E, DS> OptionValue<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: data_store::Atomic<DbError = E>,
{
    /// Updates the value in the database by applying the function `op` on it.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///     name: Option<String>,
    /// }
    ///
    /// fn first_name(s: String) -> String {
    ///     s.split_once(" ")
    ///         .map(|(first, last)| first.to_owned())
    ///         .unwrap_or(s)
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.name().set("Elijah Baley")?;
    /// db.name().update(first_name);
    /// assert_eq!(db.name().get()?, Some("Elijah".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn update(&self, op: impl FnMut(T) -> T + Clone) -> Result<(), Error<E>> {
        self.ds.atomic_update(&self.key, op)?;
        Ok(())
    }

    /// Set the value in the database to new if it is currently old.
    ///
    /// The arguments may be any borrowed form of value type, but the
    /// serialized form must match that of the value type.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///     name: Option<String>,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.name().set("Artemis")?;
    /// db.name().conditional_update("Artemis", "Helios");
    /// assert_eq!(db.name().get()?, Some("Helios".to_owned()));
    /// db.name().conditional_update("Artemis", "Zeus");
    /// assert_eq!(db.name().get()?, Some("Helios".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn conditional_update<Q>(&self, old: &Q, new: &Q) -> Result<(), Error<E>>
    where
        T: Borrow<Q>,
        Q: Serialize + ?Sized,
    {
        self.ds.conditional_update(&self.key, &new, &old)
    }
}

impl<T, E, DS> fmt::Debug for OptionValue<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned + fmt::Debug,
    DS: DataStore<DbError = E>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.get()))
    }
}
