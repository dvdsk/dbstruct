use core::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::traits::DataStore;
use crate::Error;

mod iterator;
mod extend;

/// mimics the API of [`Vec`]
pub struct Vec<T, DS>
where
    DS: DataStore,
{
    phantom: PhantomData<T>,
    ds: DS,
    prefix: u8,
    len: Arc<AtomicUsize>,
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Debug)]
pub struct Prefixed {
    prefix: u8,
    index: usize,
}

impl Prefixed {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn max(prefix: u8) -> Self {
        Self {
            prefix,
            index: usize::max_value(),
        }
    }
}

impl<T, E, DS> Vec<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<Error = E>,
{
    #[doc(hidden)]
    pub fn new(ds: DS, prefix: u8, len: Arc<AtomicUsize>) -> Self {
        Self {
            phantom: PhantomData,
            ds,
            prefix,
            len,
        }
    }

    pub fn get(&self, index: usize) -> Result<Option<T>, Error<E>> {
        let len = self.len();
        if index >= len {
            return Ok(None);
        }

        let key = Prefixed {
            prefix: self.prefix,
            index,
        };
        Ok(self.ds.get(&key)?)
    }

    pub fn push(&self, value: &T) -> Result<(), Error<E>> {
        let prev_len = self.len.fetch_add(1, Ordering::SeqCst);
        let key = Prefixed {
            prefix: self.prefix,
            index: prev_len,
        };
        debug!("pushing onto vector (index: {prev_len})");
        self.ds.insert(&key, value)?;
        Ok(())
    }

    pub fn pop(&self) -> Result<Option<T>, Error<E>> {
        let old_len = self
            .len
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |len| {
                Some(len.saturating_sub(1))
            }).expect("closure never returns None");

        let index = match old_len.checked_sub(1) {
            Some(idx) => idx,
            None => return Ok(None),
        };
        let key = Prefixed {
            prefix: self.prefix,
            index,
        };

        debug!("popping from vector (index: {index})");
        Ok(self.ds.remove(&key)?)
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores;

    pub(crate) type TestVec<T> = Vec<T, stores::BTreeMap>;
    pub(crate) fn empty<T: Clone + Serialize + DeserializeOwned>() -> TestVec<T> {
        let ds = stores::BTreeMap::new();
        let len = Arc::new(AtomicUsize::new(0));
        let vec = Vec::new(ds, 1, len);
        vec
    }

    mod given_empty_vec {
        use super::*;

        #[test]
        fn len_is_zero() {
            let vec: TestVec<()> = empty();
            assert_eq!(vec.len(), 0);
        }

        #[test]
        fn push_increases_the_len() {
            let vec = empty();
            vec.push(&42).unwrap();
            assert_eq!(vec.len(), 1)
        }

        #[test]
        fn pop_return_none() {
            let vec: TestVec<()> = empty();
            let elem = vec.pop().unwrap();
            assert_eq!(elem, None)
        }
    }

    mod given_2_long_vec {
        use super::*;

        #[test]
        fn element_pop_in_the_right_order() {
            let vec = empty();
            vec.push(&42).unwrap();
            vec.push(&43).unwrap();

            assert_eq!(vec.pop().unwrap(), Some(43));
            assert_eq!(vec.pop().unwrap(), Some(42));
        }

        #[test]
        fn third_pop_is_none() {
            let vec = empty();
            vec.push(&42).unwrap();
            vec.push(&43).unwrap();

            vec.pop().unwrap();
            vec.pop().unwrap();
            let elem = vec.pop().unwrap();
            assert_eq!(elem, None)
        }
    }
}
