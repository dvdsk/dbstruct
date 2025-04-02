use core::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::traits::DataStore;
use crate::Error;

use super::PhantomUnsync;

mod extend;
mod iterator;

/// mimics the API of [`Vec`]
pub struct Vec<T, DS>
where
    DS: DataStore,
{
    phantom: PhantomData<T>,
    phantom2: PhantomUnsync,
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
            index: usize::MAX,
        }
    }
}

impl<T, E, DS> Vec<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    #[doc(hidden)]
    pub fn new(ds: DS, prefix: u8, len: Arc<AtomicUsize>) -> Self {
        Self {
            phantom: PhantomData,
            phantom2: PhantomData,
            ds,
            prefix,
            len,
        }
    }

    /// Returns the element at `index` if there is one.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list: Vec<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert_eq!(db.list().get(0)?, None);
    /// db.list().push("a")?;
    /// db.list().push("b")?;
    /// assert_eq!(db.list().get(0)?, Some("a".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, index: usize) -> Result<Option<T>, Error<E>> {
        let len = self.len();
        if index >= len {
            return Ok(None);
        }

        let key = Prefixed {
            prefix: self.prefix,
            index,
        };
        self.ds.get(&key)
    }

    /// Appends an element to the back of the collection.
    ///
    /// The item may be any borrowed form of the lists item type, but the
    /// serialized form must match the not borrowed serialized form.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list: Vec<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().push("a")?;
    /// db.list().push("b")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push<Q>(&self, value: &Q) -> Result<(), Error<E>>
    where
        T: std::borrow::Borrow<Q>,
        Q: Serialize + ?Sized,
    {
        let prev_len = self.len.fetch_add(1, Ordering::SeqCst);
        let key = Prefixed {
            prefix: self.prefix,
            index: prev_len,
        };
        debug!("pushing onto vector (index: {prev_len})");
        self.ds.insert::<Prefixed, Q, T>(&key, value)?;
        Ok(())
    }

    /// Removes the last element from this database vector and returns it,
    /// or `None` if it is empty
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list: Vec<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().extend(["a", "b", "c"])?;
    /// assert_eq!(db.list().pop()?, Some("c".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn pop(&self) -> Result<Option<T>, Error<E>> {
        let old_len = self
            .len
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |len| {
                Some(len.saturating_sub(1))
            })
            .expect("closure never returns None");

        let index = match old_len.checked_sub(1) {
            Some(idx) => idx,
            None => return Ok(None),
        };
        let key = Prefixed {
            prefix: self.prefix,
            index,
        };

        debug!("popping from vector (index: {index})");
        self.ds.remove(&key)
    }

    /// Clears the list, removing all values.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list: Vec<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().extend(["a", "b", "c"])?;
    /// assert!(!db.list().is_empty());
    /// db.list().clear();
    /// assert!(db.list().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear(&self) -> Result<(), Error<E>> {
        for _ in 0..self.len() {
            self.pop()?;
        }
        Ok(())
    }

    /// Returns the number of elements in the list, also referred to as its 'length'.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list: Vec<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().extend(["a", "b", "c"])?;
    /// assert_eq!(db.list().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    /// Returns `true` if the list has a length of 0.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list_a: Vec<String>,
    ///	    list_b: Vec<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list_a().extend(["a", "b", "c"])?;
    /// assert!(!db.list_a().is_empty());
    /// assert!(db.list_b().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T, E, DS> fmt::Debug for Vec<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned + fmt::Debug,
    DS: DataStore<DbError = E>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[\n")?;
        for element in self.iter() {
            match element {
                Ok(val) => f.write_fmt(format_args!("    {val:?},\n"))?,
                Err(err) => {
                    f.write_fmt(format_args!(
                        "ERROR while printing full list, could \
                         not read next element from db: {err}"
                    ))?;
                    return Ok(());
                }
            }
        }
        f.write_str("]\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores;

    pub(crate) type TestVec<T> = Vec<T, stores::BTreeMap>;
    pub(crate) fn empty<T: Serialize + DeserializeOwned>() -> TestVec<T> {
        let ds = stores::BTreeMap::new();
        let len = Arc::new(AtomicUsize::new(0));

        Vec::new(ds, 1, len)
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
            let vec: Vec<u16, _> = empty();
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
            let vec: Vec<u16, stores::BTreeMap> = empty();
            vec.push(&42).unwrap();
            vec.push(&43).unwrap();

            vec.pop().unwrap();
            vec.pop().unwrap();
            let elem = vec.pop().unwrap();
            assert_eq!(elem, None)
        }
    }
}
