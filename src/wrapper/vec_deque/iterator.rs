use super::VecDeque;
use crate::Error;
use std::fmt;

use crate::traits::DataStore;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Iter<'a, T, E, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    pub(crate) current: usize,
    pub(crate) deque: &'a VecDeque<T, DS>,
}

impl<T, E, DS> Iterator for Iter<'_, T, E, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    type Item = Result<T, Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        let elem = self.deque.get(self.current);
        self.current += 1;
        elem.transpose()
    }
}

/// This can be quite slow as it gets each element from
/// the db individually. Consider using the Default wrapper
/// instead of this if the `VecDeque` is "small" enough.
impl<'a, T, E, DS> IntoIterator for &'a VecDeque<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    type IntoIter = Iter<'a, T, E, DS>;
    type Item = Result<T, Error<E>>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            current: 0,
            deque: self,
        }
    }
}

/// This can be quite slow as it gets each element from
/// the db individually. Consider using the Default wrapper
/// instead of this if the `VecDeque` is "small" enough.
impl<'a, T, E, DS> VecDeque<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    pub fn iter(&self) -> Iter<T, E, DS> {
        Iter {
            current: 0,
            deque: self,
        }
    }
}
