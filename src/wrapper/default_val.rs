use std::fmt;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::{data_store, DataStore};
use crate::Error;

/// handles missing values by generating a replacement from an expression. 
pub struct DefaultValue<T, DS>
where
    T: Serialize + DeserializeOwned + Clone,
    DS: DataStore,
{
    default_value: T,
    ds: DS,
    key: u8,
}

impl<T, E, DS> DefaultValue<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned + Clone,
    DS: DataStore<DbError = E>,
{
    #[doc(hidden)]
    pub fn new(ds: DS, key: u8, default_value: T) -> Self {
        Self {
            default_value,
            ds,
            key,
        }
    }

    pub fn set(&mut self, value: &T) -> Result<(), Error<E>> {
        self.ds.insert::<_, T, T>(&self.key, value)?;
        Ok(())
    }

    pub fn get(&self) -> Result<T, Error<E>> {
        Ok(self
            .ds
            .get(&self.key)?
            .unwrap_or_else(|| self.default_value.clone()))
    }
}

impl<T, E, DS> DefaultValue<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned + Clone,
    DS: data_store::Atomic<DbError = E>,
{
    pub fn update(&self, op: impl FnMut(T) -> T + Clone) -> Result<(), Error<E>> {
        self.ds.atomic_update(&self.key, op)?;
        Ok(())
    }
    pub fn conditional_update(&self, old: T, new: T) -> Result<(), Error<E>> {
        self.ds.conditional_update(&self.key, &new, &old)
    }
}
