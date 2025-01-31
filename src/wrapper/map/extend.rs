use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::ExtendError;
use crate::{DataStore, TryExtend};

use super::Map;

/// Inserts all new key-values from the iterator and replaces values with
/// existing keys with new values returned from the iterator.
impl<Key, Value, DS> TryExtend<(Key, Value)> for Map<'_, Key, Value, DS>
where
    DS: DataStore,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
{
    type DbError = DS::DbError;

    fn try_extend<I>(
        &mut self,
        iter: I,
    ) -> Result<(), ExtendError<I::Item, I::IntoIter, crate::Error<DS::DbError>>>
    where
        I: IntoIterator<Item = (Key, Value)>,
    {
        let mut iter = iter.into_iter();
        loop {
            let Some((key, value)) = iter.next() else {
                return Ok(());
            };

            if let Err(error) = self.insert(&key, &value) {
                return Err(ExtendError {
                    unadded: (key, value),
                    iter,
                    error,
                });
            }
        }
    }
}

/// Inserts all new key-values from the iterator and replaces values with
/// existing keys with new values returned from the iterator.
impl<'a, Key, Value, DS> TryExtend<(&'a Key, &'a Value)> for Map<'_, Key, Value, DS>
where
    DS: DataStore,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
{
    type DbError = DS::DbError;

    fn try_extend<I>(
        &mut self,
        iter: I,
    ) -> Result<(), ExtendError<I::Item, I::IntoIter, crate::Error<Self::DbError>>>
    where
        I: IntoIterator<Item = (&'a Key, &'a Value)>,
    {
        let mut iter = iter.into_iter();
        loop {
            let Some((key, value)) = iter.next() else {
                return Ok(());
            };

            if let Err(error) = self.insert(&key, &value) {
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
            let mut map = Map::new(ds.clone(), 1);

            let iter = [("a".to_string(), 1)];
            ds.force_error();
            let err = map
                .try_extend(iter)
                .expect_err("we forced the datastore to crash on access");
            assert_eq!(err.unadded, ("a".to_string(), 1));
            assert_eq!(err.iter.collect::<std::vec::Vec<_>>(), vec![]);
        }

        #[test]
        fn without_error() {
            let ds = stores::BTreeMap::new();
            let mut map: Map<String, usize, _> = Map::new(ds.clone(), 1);

            let iter = [(&"a".to_string(), &1)];
            map.try_extend(iter).unwrap();
            let res: Result<Vec<(_, _)>, _> = map.iter().collect();
            assert_eq!(res.unwrap(), vec![("a".to_string(), 1)]);
        }
    }
}
