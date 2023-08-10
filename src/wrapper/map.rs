use core::fmt;
use std::marker::PhantomData;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{instrument, trace};

use crate::traits::DataStore;
use crate::Error;

mod iterator;

/// mimics the API of [`HashMap`][std::collections::HashMap]
pub struct Map<'a, Key, Value, DS>
where
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore,
{
    phantom_key: PhantomData<&'a Key>,
    phantom_val: PhantomData<Value>,
    tree: DS,
    prefix: u8,
}

#[derive(Serialize)]
pub struct Prefixed<'a, K> {
    prefix: u8,
    key: &'a K,
}

impl<'a, Key, Value, E, DS> Map<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<Error = E>,
{
    #[doc(hidden)]
    #[instrument(skip(tree), level = "debug")]
    pub fn new(tree: DS, prefix: u8) -> Self {
        Self {
            phantom_key: PhantomData,
            phantom_val: PhantomData,
            tree,
            prefix,
        }
    }

    fn prefix(&self, key: &'a Key) -> Prefixed<'a, Key> {
        trace!("prefixing key with: {}", self.prefix);
        Prefixed {
            prefix: self.prefix,
            key,
        }
    }

    /// returns existing value if any was set
    #[instrument(skip_all, level = "debug")]
    pub fn insert(&self, key: &'a Key, value: &'a Value) -> Result<Option<Value>, Error<E>> {
        let key = self.prefix(key);
        let existing = self.tree.insert(&key, value).unwrap();
        Ok(existing)
    }

    #[instrument(skip_all, level = "debug")]
    pub fn get(&self, key: &'a Key) -> Result<Option<Value>, Error<E>> {
        let key = self.prefix(key);
        let value = self.tree.get(&key).unwrap();
        Ok(value)
    }
}
