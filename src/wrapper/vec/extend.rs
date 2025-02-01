use std::borrow::Borrow;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::ExtendError;
use crate::DataStore;

use super::Vec;

impl<'a, T, DS> Vec<T, DS>
where
    DS: DataStore,
    T: Serialize + DeserializeOwned,
{
    /// Extends the list with the contents of an iterator.
    ///
    /// The iterator item may be any borrowed form of the lists item type, 
    /// as long as the serialized form matches between borrowed and not borrowed.
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
    /// db.list().extend(["one", "two", "three"])?;
    /// assert_eq!(db.list().get(0)?, Some("one".to_owned()));
    /// assert_eq!(db.list().get(2)?, Some("three".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn extend<I, Q>(
        &mut self,
        iter: I,
    ) -> Result<(), ExtendError<I::Item, I::IntoIter, crate::Error<DS::DbError>>>
    where
        I: IntoIterator<Item = &'a Q>,
        T: Borrow<Q>,
        Q: Serialize + ?Sized,
    {
        let mut iter = iter.into_iter();
        loop {
            let Some(item) = iter.next() else {
                return Ok(());
            };

            if let Err(error) = self.push::<Q>(item) {
                return Err(ExtendError {
                    unadded: item,
                    iter,
                    error,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stores;
    use crate::wrapper::Vec;
    use std::sync::{atomic::AtomicUsize, Arc};

    #[test]
    fn error() {
        let ds = stores::BTreeMap::new();
        let len = Arc::new(AtomicUsize::new(0));
        let mut vec: Vec<u16, stores::BTreeMap> = Vec::new(ds.clone(), 1, len);

        let iter = [1, 2, 3, 4];
        ds.force_error();
        let err = vec
            .extend(&iter)
            .expect_err("we forced the datastore to crash on access");
        assert_eq!(err.unadded, &1);
        assert_eq!(err.iter.collect::<std::vec::Vec<_>>(), vec![&2, &3, &4]);
    }

    #[test]
    fn push_str_slices() {
        let ds = stores::BTreeMap::new();
        let len = Arc::new(AtomicUsize::new(0));
        let mut vec: Vec<String, _> = Vec::new(ds.clone(), 1, len);

        let iter = ["1", "2", "3", "4"];
        vec.extend(iter).unwrap();
        assert_eq!(vec.len(), 4);
    }

    #[test]
    fn push_strings() {
        let ds = stores::BTreeMap::new();
        let len = Arc::new(AtomicUsize::new(0));
        let mut vec: Vec<String, _> = Vec::new(ds.clone(), 1, len);

        let iter = [
            "1".to_owned(),
            "2".to_owned(),
            "3".to_owned(),
            "4".to_owned(),
        ];
        vec.extend(&iter).unwrap();
    }
}
