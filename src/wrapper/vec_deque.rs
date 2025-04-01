use core::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::u64;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::traits::DataStore;
use crate::Error;

mod extend;
mod iterator;

/// mimics the API of [`VecDeque`](std::collections::VecDeque)
pub struct VecDeque<T, DS>
where
    DS: DataStore,
{
    phantom: PhantomData<T>,
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
        assert_ne!(head.load(Ordering::Relaxed), tail.load(Ordering::Relaxed));
        Self {
            phantom: PhantomData,
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
        if index >= self.len() as usize {
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
    ///	    list: Vec<String>,
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

        eprintln!(
            "push_back, {:?}..{:?}, idx: {:?}",
            &self.head, &self.tail, key.index
        );
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
    ///	    list: Vec<String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().front_back("b")?;
    /// db.list().front_back("a")?;
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

        eprintln!(
            "push_frnt, {:?}..{:?}, idx: {:?}",
            &self.head, &self.tail, key.index
        );
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
    ///	    list: Vec<String>,
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
        let free_tail = self.tail.fetch_sub(1, Ordering::Relaxed);
        let key = Prefixed {
            prefix: self.prefix,
            index: free_tail - 1,
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
    ///	    list: Vec<String>,
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
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);

        return ((tail - head) - 1) as usize;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores;

    pub(crate) type TestVecDeque<T> = VecDeque<T, stores::BTreeMap>;
    pub(crate) fn empty<T: Serialize + DeserializeOwned>() -> TestVecDeque<T> {
        let ds = stores::BTreeMap::new();
        let head = Arc::new(AtomicU64::new(u64::MAX / 2 - 1));
        let tail = Arc::new(AtomicU64::new(u64::MAX / 2));

        VecDeque::new(ds, 1, head, tail)
    }

    mod given_empty {
        use super::*;

        #[test]
        fn len_is_zero() {
            let vec: TestVecDeque<()> = empty();
            assert_eq!(vec.len(), 0);
        }

        #[test]
        fn push_increases_the_len() {
            let vec: VecDeque<u16, _> = empty();
            vec.push_back(&42).unwrap();
            assert_eq!(vec.len(), 1)
        }

        #[test]
        fn pop_return_none() {
            let vec: TestVecDeque<()> = empty();
            let elem = vec.pop_back().unwrap();
            assert_eq!(elem, None)
        }
    }

    mod push_pop_back {
        use super::*;

        #[test]
        fn len_is_two() {
            let vec: VecDeque<i32, _> = empty();
            vec.push_back(&42).unwrap();
            vec.push_back(&43).unwrap();

            assert_eq!(vec.len(), 2);
        }

        #[test]
        fn element_pop_in_the_right_order() {
            let vec = empty();
            vec.push_back(&42).unwrap();
            vec.push_back(&43).unwrap();

            assert_eq!(vec.pop_back().unwrap(), Some(43));
            assert_eq!(vec.pop_back().unwrap(), Some(42));
        }

        #[test]
        fn third_pop_is_none() {
            let vec: VecDeque<u16, stores::BTreeMap> = empty();
            vec.push_back(&42).unwrap();
            vec.push_back(&43).unwrap();

            vec.pop_back().unwrap();
            vec.pop_back().unwrap();
            let elem = vec.pop_back().unwrap();
            assert_eq!(elem, None)
        }
    }

    mod push_pop_front {
        use super::*;

        #[test]
        fn len_is_two() {
            let vec: VecDeque<i32, _> = empty();
            vec.push_front(&42).unwrap();
            vec.push_front(&43).unwrap();

            assert_eq!(vec.len(), 2);
        }

        #[test]
        fn element_pop_in_the_right_order() {
            let vec = empty();
            vec.push_front(&42).unwrap();
            vec.push_front(&43).unwrap();

            assert_eq!(vec.pop_front().unwrap(), Some(43));
            assert_eq!(vec.pop_front().unwrap(), Some(42));
        }

        #[test]
        fn third_pop_is_none() {
            let vec: VecDeque<u16, stores::BTreeMap> = empty();
            vec.push_front(&42).unwrap();
            vec.push_front(&43).unwrap();

            vec.pop_front().unwrap();
            vec.pop_front().unwrap();
            let elem = vec.pop_front().unwrap();
            assert_eq!(elem, None)
        }
    }

    mod combined_front_back {
        use super::*;

        #[test]
        fn len_is_correct() {
            let vec: VecDeque<i32, _> = empty();
            vec.push_front(&42).unwrap();
            assert_eq!(vec.len(), 1);

            vec.push_back(&43).unwrap();
            assert_eq!(vec.len(), 2);

            vec.push_front(&41).unwrap();
            assert_eq!(vec.len(), 3);
        }

        #[test]
        fn element_pop_in_the_right_order() {
            let vec = empty();
            vec.push_front(&42).unwrap();
            vec.push_back(&43).unwrap();
            vec.push_front(&41).unwrap();

            assert_eq!(vec.pop_front().unwrap(), Some(41));
            assert_eq!(vec.pop_back().unwrap(), Some(43));
            assert_eq!(vec.pop_front().unwrap(), Some(42));
        }

        #[test]
        fn fourth_pop_is_none() {
            let vec: VecDeque<u16, stores::BTreeMap> = empty();
            vec.push_front(&42).unwrap();
            vec.push_back(&43).unwrap();
            vec.push_front(&41).unwrap();

            vec.pop_front().unwrap();
            vec.pop_front().unwrap();
            vec.pop_front().unwrap();
            let elem = vec.pop_front().unwrap();
            assert_eq!(elem, None)
        }

        #[test]
        fn access_front_via_back_pop() {
            let vec: VecDeque<u16, stores::BTreeMap> = empty();
            vec.push_front(&43).unwrap();
            vec.push_front(&42).unwrap();
            vec.push_front(&41).unwrap();

            assert_eq!(vec.pop_back().unwrap(), Some(43));
            assert_eq!(vec.pop_back().unwrap(), Some(42));
            assert_eq!(vec.pop_back().unwrap(), Some(41));
        }
    }
}
