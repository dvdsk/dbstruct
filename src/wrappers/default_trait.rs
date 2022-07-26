use core::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Error;
use crate::traits::DataStore;

pub struct DefaultTrait<T, DS> 
where 
    T: Serialize + DeserializeOwned,
    DS: DataStore<u8, T>
{
    phantom: PhantomData<T>,
    ds: DS,
    key: u8,
}

impl<T, E, DS> DefaultTrait<T, DS>
where
    E: fmt::Debug,
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

    pub fn set(&mut self, value: &T) -> Result<(), Error<E>> {
        self.ds.insert(&self.key, value)?;
        Ok(())
    }

    pub fn get(&self) -> Result<T, Error<E>> {
        Ok(self.ds.get(&self.key)?.unwrap_or_default())
    }
}
