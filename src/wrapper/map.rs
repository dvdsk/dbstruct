use core::fmt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use tracing::{instrument, trace};

use crate::traits::data_store::Ranged;
use crate::traits::DataStore;

mod extend;
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
    pub fn insert(&self, key: &'a Key, value: &'a Value) -> Result<Option<Value>, E> {
        let key = self.prefix(key);
        let existing = self.tree.insert(&key, value)?;
        Ok(existing)
    }

    #[instrument(skip_all, level = "debug")]
    pub fn get(&self, key: &'a Key) -> Result<Option<Value>, E> {
        let key = self.prefix(key);
        let value = self.tree.get(&key)?;
        Ok(value)
    }

    #[instrument(skip_all, level = "debug")]
    pub fn remove(&self, key: &'a Key) -> Result<Option<Value>, E> {
        let key = self.prefix(key);
        let value = self.tree.remove(&key)?;
        Ok(value)
    }
}

impl<'a, Key, Value, E, DS> Map<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: Ranged<Error = E>,
{
    pub fn clear(&self) -> Result<(), E> {
        let first = self.prefix;
        let after_last = self.prefix + 1;
        let iter = self.tree.range::<_, _, Value>(first..after_last)?;
        for res in iter {
            let (key, _) = res?;
            self.remove(&key)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores;

    pub(crate) type TestMap<'a, K, V> = Map<'a, K, V, stores::BTreeMap>;
    pub(crate) fn empty<'a, K, V>() -> TestMap<'a, K, V>
    where
        K: Clone + Serialize + DeserializeOwned,
        V: Clone + Serialize + DeserializeOwned,
    {
        let ds = stores::BTreeMap::new();
        let map = Map::new(ds, 1);
        map
    }
}
