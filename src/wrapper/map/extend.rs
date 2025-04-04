use std::borrow::Borrow;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::ExtendError;
use crate::DataStore;

use super::Map;

/// Inserts all new key-values from the iterator and replaces values with
/// existing keys with new values returned from the iterator.
impl<Key, Value, DS> Map<Key, Value, DS>
where
    DS: DataStore,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
{
    /// Extends the map with the contents of an iterator of tuples.
    ///
    /// The key and value in the tuple may be any borrowed form. As long as
    /// the serialized form matches between borrowed and not borrowed.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///     map: HashMap<u16, String>,
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().extend([(&1, "one"), (&2, "two"), (&3, "three")])?;
    /// assert_eq!(db.map().get(&1)?, Some("one".to_owned()));
    /// assert_eq!(db.map().get(&3)?, Some("three".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn extend<'a, I, K, V>(
        &mut self,
        iter: I,
    ) -> Result<(), ExtendError<I::Item, I::IntoIter, crate::Error<DS::DbError>>>
    where
        I: IntoIterator<Item = (&'a K, &'a V)>,
        Key: Borrow<K>,
        K: Serialize + ?Sized,
        Value: Borrow<V>,
        V: Serialize + ?Sized,
    {
        let mut iter = iter.into_iter();
        loop {
            let Some((key, value)) = iter.next() else {
                return Ok(());
            };

            if let Err(error) = self.insert(key, value) {
                return Err(ExtendError {
                    unadded: (key, value),
                    iter,
                    error,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores;

    mod iter_one_item {
        use super::*;

        #[test]
        fn while_db_errors() {
            let ds = stores::BTreeMap::new();
            let mut map: Map<String, u16, _> = Map::new(ds.clone(), 1);

            let iter = [("a", &1)];
            ds.force_error();
            let err = map
                .extend(iter)
                .expect_err("we forced the datastore to crash on access");
            assert_eq!(err.unadded, ("a", &1));
            assert_eq!(err.iter.collect::<std::vec::Vec<_>>(), vec![]);
        }

        #[test]
        fn without_error() {
            let ds = stores::BTreeMap::new();
            let mut map: Map<String, usize, _> = Map::new(ds.clone(), 1);

            let iter = [(&"a".to_string(), &1)];
            map.extend(iter).unwrap();
            let res: Result<Vec<(_, _)>, _> = map.iter().collect();
            assert_eq!(res.unwrap(), vec![("a".to_string(), 1)]);
        }
    }
}
