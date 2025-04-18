use core::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::DataStore;
use crate::Error;

/// handles missing values by generating a replacement using the types [`Default`] implementation
pub struct DefaultTrait<T, DS>
where
    T: Serialize + DeserializeOwned + Default,
    DS: DataStore,
{
    phantom: PhantomData<T>,
    ds: DS,
    key: u8,
}

impl<T, E, DS> DefaultTrait<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned + Default,
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
        self.ds.insert::<_, T, T>(&self.key, value)?;
        Ok(())
    }

    pub fn get(&self) -> Result<T, Error<E>> {
        Ok(self.ds.get(&self.key)?.unwrap_or_default())
    }
}
