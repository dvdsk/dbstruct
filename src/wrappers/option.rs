use core::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::{data_store, DataStore}; 
use crate::Error;

// while defaultvalue requires T: Clone, this does not
pub struct OptionValue<T, DS>
where
    T: Serialize + DeserializeOwned,
    DS: DataStore<u8, T>,
{
    phantom: PhantomData<T>,
    ds: DS,
    key: u8,
}

impl<T, E, DS> OptionValue<T, DS>
where
    E: fmt::Debug,
    Error: From<E>,
    T: Serialize + DeserializeOwned + Default,
    DS: DataStore<u8, T, Error = E>,
{
    pub fn new(ds: DS, key: u8) -> Self {
        Self {
            phantom: PhantomData::default(),
            ds,
            key,
        }
    }

    pub fn set(&mut self, value: &T) -> Result<(), Error> {
        self.ds.insert(&self.key, value)?;
        Ok(())
    }

    pub fn get(&self) -> Result<Option<T>, Error> {
        Ok(self.ds.get(&self.key)?)
    }

    /// if the value is None then no update is performed
    pub fn conditional_update(&self, old: T, new: T) -> Result<(), Error> {
        Ok(self.ds.conditional_update(&self.key, &new, &old)?)
    }
}

impl<T, E, DS> OptionValue<T, DS>
where
    E: fmt::Debug,
    Error: From<E>,
    T: Serialize + DeserializeOwned + Default,
    DS: data_store::Atomic<u8, T, Error = E>,
{
    fn update(&self, op: impl FnMut(T) -> T + Clone) -> Result<(), Error> {
        self.ds.atomic_update(&self.key, op)?;
        Ok(())
    }
}
