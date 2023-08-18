use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::ExtendError;
use crate::{DataStore, TryExtend};

use super::Vec;

/// Pushes the values from the iterator onto the vec.
///
/// # Note
/// Does not guarentee the content of the iterator is pushed 
/// atomically. Parallel access could intersperse items.
impl<T, DS> TryExtend<T> for Vec<T, DS>
where
    DS: DataStore,
    T: Serialize + DeserializeOwned,
{
    type Error = crate::Error<DS::Error>;

    fn try_extend<I>(&mut self, iter: I) -> Result<(), ExtendError<T, I::IntoIter, Self::Error>>
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        loop {
            let Some(item) = iter.next() else {
                return Ok(());
            };

            if let Err(error) = self.push(&item) {
                return Err(ExtendError {
                    unadded: item,
                    iter,
                    error,
                });
            }
        }
    }
}

/// Pushes the values from the iterator onto the vec.
///
/// # Note
/// Does not guarentee the content of the iterator is pushed 
/// atomically. Parallel access could intersperse items.
impl<'a, T, DS> TryExtend<&'a T> for Vec<T, DS>
where
    DS: DataStore,
    T: Serialize + DeserializeOwned,
{
    type Error = crate::Error<DS::Error>;

    fn try_extend<I>(&mut self, iter: I) -> Result<(), ExtendError<I::Item, I::IntoIter, Self::Error>>
    where
        I: IntoIterator<Item = &'a T>,
    {
        let mut iter = iter.into_iter();
        loop {
            let Some(item) = iter.next() else {
                return Ok(());
            };

            if let Err(error) = self.push(&item) {
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
    use super::*;
    use crate::stores;
    use std::sync::{atomic::AtomicUsize, Arc};

    #[test]
    fn error() {
        let ds = stores::BTreeMap::new();
        let len = Arc::new(AtomicUsize::new(0));
        let mut vec = Vec::new(ds.clone(), 1, len);

        let iter = [1, 2, 3, 4];
        ds.force_error();
        let err = vec
            .try_extend(iter)
            .expect_err("we forced the datastore to crash on access");
        assert_eq!(err.unadded, 1);
        assert_eq!(err.iter.collect::<std::vec::Vec<_>>(), vec![2, 3, 4]);
    }
}
