use std::borrow::Borrow;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::ExtendError;
use crate::DataStore;

use super::VecDeque;

impl<'a, T, DS> VecDeque<T, DS>
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
    ///     list: Vec<String>,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.list().extend(["one", "two", "three"])?;
    /// assert_eq!(db.list().get(0)?, Some("one".to_owned()));
    /// assert_eq!(db.list().get(2)?, Some("three".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::type_complexity)]
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

            if let Err(error) = self.push_back::<Q>(item) {
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
    use crate::wrapper::VecDeque;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use std::u64;

    #[test]
    fn error() {
        let ds = stores::BTreeMap::new();
        let tail = Arc::new(AtomicU64::new(u64::MAX / 2));
        let head = Arc::new(AtomicU64::new(u64::MAX / 2 - 1));
        let mut vec: VecDeque<u16, stores::BTreeMap> = VecDeque::new(ds.clone(), 1, tail, head);

        let iter = [1, 2, 3, 4];
        ds.force_error();
        let err = vec
            .extend(&iter)
            .expect_err("we forced the datastore to crash on access");
        assert_eq!(err.unadded, &1);
        assert_eq!(
            err.iter.collect::<std::collections::VecDeque<_>>(),
            vec![&2, &3, &4]
        );
    }
}
