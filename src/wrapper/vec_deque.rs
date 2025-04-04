use core::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::u64;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::traits::DataStore;
use crate::Error;

use super::PhantomUnsync;

mod extend;
mod iterator;

/// mimics the API of [`VecDeque`](std::collections::VecDeque)
pub struct VecDeque<T, DS>
where
    DS: DataStore,
{
    phantom: PhantomData<T>,
    phantom2: PhantomUnsync,
    ds: DS,
    prefix: u8,
    // Points to the current free slot
    head: Arc<AtomicU64>,
    // Points to the current free slot
    tail: Arc<AtomicU64>,
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Debug)]
pub struct Prefixed {
    prefix: u8,
    /// Rather large so we can *just* start in the middle and be sure
    /// you can never reach an edge
    index: u64,
}

impl Prefixed {
    pub fn index(&self) -> u64 {
        self.index
    }
    pub fn min(prefix: u8) -> Self {
        Self { prefix, index: 0 }
    }
    pub fn max(prefix: u8) -> Self {
        Self {
            prefix,
            index: u64::MAX,
        }
    }
}

impl<T, E, DS> VecDeque<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    #[doc(hidden)]
    pub fn new(ds: DS, prefix: u8, head: Arc<AtomicU64>, tail: Arc<AtomicU64>) -> Self {
        assert_ne!(
            head.load(Ordering::Relaxed),
            tail.load(Ordering::Relaxed),
            "VecDeque::new failed"
        );
        Self {
            phantom: PhantomData,
            phantom2: PhantomData,
            ds,
            prefix,
            head,
            tail,
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
    ///	    list: VecDeque<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// assert_eq!(db.list().get(0)?, None);
    /// db.list().push_back("a")?;
    /// db.list().push_back("b")?;
    /// assert_eq!(db.list().get(0)?, Some("a".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, index: usize) -> Result<Option<T>, Error<E>> {
        if index >= self.len() {
            return Ok(None);
        }

        let head = self.head.load(Ordering::Relaxed);
        let db_index = index as u64 + head + 1;

        let key = Prefixed {
            prefix: self.prefix,
            index: db_index,
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
    ///	    list: VecDeque<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().push_back("a")?;
    /// db.list().push_back("b")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push_back<Q>(&self, value: &Q) -> Result<(), Error<E>>
    where
        T: std::borrow::Borrow<Q>,
        Q: Serialize + ?Sized,
    {
        let free_tail = self.tail.fetch_add(1, Ordering::SeqCst);
        let key = Prefixed {
            prefix: self.prefix,
            index: free_tail,
        };

        self.ds.insert::<Prefixed, Q, T>(&key, value)?;
        Ok(())
    }

    /// Appends an element to the front of the collection.
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
    ///	    list: VecDeque<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().push_front("b")?;
    /// db.list().push_front("a")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push_front<Q>(&self, value: &Q) -> Result<(), Error<E>>
    where
        T: std::borrow::Borrow<Q>,
        Q: Serialize + ?Sized,
    {
        let free_head = self.head.fetch_sub(1, Ordering::SeqCst);
        let key = Prefixed {
            prefix: self.prefix,
            index: free_head,
        };

        self.ds.insert::<Prefixed, Q, T>(&key, value)?;
        Ok(())
    }

    /// Removes the last element from this database deque and returns it,
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
    ///	    list: VecDeque<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().extend(["a", "b", "c"])?;
    /// assert_eq!(db.list().pop_back()?, Some("c".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn pop_back(&self) -> Result<Option<T>, Error<E>> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Relaxed);
        let next_tail = tail - 1;
        if next_tail != head {
            self.tail.store(tail - 1, Ordering::Relaxed);
        }
        // TODO TAIL AND HEAD COLLIDE, TEST THIS AND FIX POP_FRONT SIMILARLY

        let key = Prefixed {
            prefix: self.prefix,
            index: tail - 1,
        };

        self.ds.remove(&key)
    }

    /// Removes the first element from this database deque and returns it,
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
    ///	    list: VecDeque<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().extend(["a", "b", "c"])?;
    /// assert_eq!(db.list().pop_front()?, Some("a".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn pop_front(&self) -> Result<Option<T>, Error<E>> {
        let free_head = self.head.fetch_add(1, Ordering::Relaxed);
        let key = Prefixed {
            prefix: self.prefix,
            index: free_head + 1,
        };

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
    ///	    list: VecDeque<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().extend(["a", "b", "c"])?;
    /// assert!(!db.list().is_empty());
    /// db.list().clear();
    /// dbg!("post_clear");
    /// assert!(db.list().is_empty());
    /// dbg!("post fn");
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear(&self) -> Result<(), Error<E>> {
        // Keep going back until nothing left, as long as pop_back works
        // concurrently this is safe too
        while self.pop_back()?.is_some() {}
        Ok(())
    }

    /// Returns the number of elements in the list, also referred to as its 'length'.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list: VecDeque<String>,
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
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);

        ((tail - head) - 1) as usize
    }

    /// Returns `true` if the list has a length of 0.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    list_a: VecDeque<String>,
    ///	    list_b: VecDeque<String>,
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

impl<T, E, DS> fmt::Debug for VecDeque<T, DS>
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
