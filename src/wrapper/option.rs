use core::fmt;
use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::traits::{data_store, DataStore};
use crate::Error;

/// here missing values are represented by [`Option::None`]. 
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

    pub fn set(&mut self, value: &T) -> Result<(), Error<E>> {
        self.ds.insert(&self.key, value)?;
        Ok(())
    }

    pub fn get(&self) -> Result<Option<T>, Error<E>> {
        self.ds.get(&self.key)
    }
}

impl<T, E, DS> OptionValue<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: data_store::Atomic<DbError = E>,
{
    pub fn update(&self, op: impl FnMut(T) -> T + Clone) -> Result<(), Error<E>> {
        self.ds.atomic_update(&self.key, op)?;
        Ok(())
    }
    /// if the value is None then no update is performed
    pub fn conditional_update(&self, old: T, new: T) -> Result<(), Error<E>> {
        Ok(self.ds.conditional_update(&self.key, &new, &old)?)
    }
}
