use core::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::DataStore;
use crate::Error;

pub struct Vec<T, DS>
where
    T: Serialize + DeserializeOwned,
    DS: DataStore<Prefixed, T>,
{
    phantom: PhantomData<T>,
    ds: DS,
    prefix: u8,
    len: Arc<AtomicUsize>,
}

#[derive(Serialize)]
pub struct Prefixed {
    prefix: u8,
    index: usize,
}

// probably gonna need to add a method to keep the index for the Vec wrapper
// might need to scan on creation (::new), could also specialize that to use less then
// when supported (sled etc). TODO figure out a way to represent that in a trait
impl<T, E, DS> Vec<T, DS>
where
    E: fmt::Debug,
    Error: From<E>,
    T: Serialize + DeserializeOwned,
    DS: DataStore<Prefixed, T, Error = E>,
{
    pub fn new(ds: DS, prefix: u8, len: Arc<AtomicUsize>) -> Self {
        Self {
            phantom: PhantomData,
            ds,
            prefix,
            len,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), Error> {
        let new_len = self.len.fetch_add(1, Ordering::SeqCst);
        let key = Prefixed {
            prefix: self.prefix,
            index: new_len - 1,
        };
        self.ds.insert(&key, &value)?;
        Ok(())
    }

    pub fn pop(&self) -> Result<Option<T>, Error> {
        let new_len = self.len.fetch_sub(1, Ordering::SeqCst);
        let key = Prefixed {
            prefix: self.prefix,
            index: new_len,
        };

        Ok(self.ds.remove(&key)?)
    }
}
