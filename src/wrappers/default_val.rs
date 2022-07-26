use std::fmt;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::{data_store, DataStore};
use crate::Error;

pub struct DefaultValue<T, DS>
where
    T: Serialize + DeserializeOwned + Clone,
    DS: DataStore<u8, T>,
{
    default_value: T,
    ds: DS,
    key: u8,
}

impl<T, E, DS> DefaultValue<T, DS>
where
    E: fmt::Debug,
    Error: From<E>,
    T: Serialize + DeserializeOwned + Clone,
    DS: DataStore<u8, T, Error = E>,
{
    pub fn new(ds: DS, key: u8, default_value: T) -> Self {
        Self {
            default_value,
            ds,
            key,
        }
    }

    pub fn set(&mut self, value: &T) -> Result<(), Error> {
        self.ds.insert(&self.key, value)?;
        Ok(())
    }

    pub fn get(&self) -> Result<T, Error> {
        Ok(self
            .ds
            .get(&self.key)?
            .unwrap_or_else(|| self.default_value.clone()))
    }
}

impl<T, E, DS> DefaultValue<T, DS>
where
    E: fmt::Debug,
    Error: From<E>,
    T: Serialize + DeserializeOwned + Clone,
    DS: data_store::Atomic<u8, T, Error = E>,
{
    pub fn update(&self, op: impl FnMut(T) -> T + Clone) -> Result<(), Error> {
        self.ds.atomic_update(&self.key, op)?;
        Ok(())
    }
    pub fn conditional_update(&self, old: T, new: T) -> Result<(), Error> {
        Ok(self.ds.conditional_update(&self.key, &new, &old)?)
    }
}
