use core::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::DataStore;
use crate::Error;

pub struct Map<'a, Key, Value, DS>
where
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<Prefixed<'a, Key>, Value>,
{
    phantom_key: PhantomData<&'a Key>,
    phantom_val: PhantomData<Value>,
    tree: DS,
    prefix: u8,
}

#[derive(Serialize)]
pub struct Prefixed<'a, K>
where
    K: Serialize,
{
    prefix: u8,
    key: &'a K,
}

impl<'a, Key, Value, E, DS> Map<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Error: From<E>,
    Key: Serialize,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<Prefixed<'a, Key>, Value, Error = E>,
{
    pub fn new(tree: DS, prefix: u8) -> Self {
        Self {
            phantom_key: PhantomData,
            phantom_val: PhantomData,
            tree,
            prefix,
        }
    }

    // TODO! Figure out a way to use DataStore trait with prefixes
    // probably gonna need to add a method to keep the index for the Vec wrapper
    // might need to scan on creation (::new), could also specialize that to use less then
    // when supported (sled etc). TODO figure out a way to represent that in a trait
    //
    /// returns existing value if any was set
    pub fn set(&self, key: &'a Key, value: &'a Value) -> Result<Option<Value>, Error> {
        let key = Prefixed {
            prefix: self.prefix,
            key,
        };
        let existing = self.tree.insert(&key, value).unwrap();
        Ok(existing)
    }

    pub fn get(&self, key: &'a Key) -> Result<Option<Value>, Error> {
        let key = Prefixed {
            prefix: self.prefix,
            key,
        };
        let value = self.tree.get(&key).unwrap();
        Ok(value)
    }
}
