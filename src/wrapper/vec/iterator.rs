use super::Vec;
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
    pub(crate) vec: &'a Vec<T, DS>,
}

impl<T, E, DS> Iterator for Iter<'_, T, E, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    type Item = Result<T, Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        let elem = self.vec.get(self.current);
        self.current += 1;
        elem.transpose()
    }
}

/// This can be quite slow as it gets each element from
/// the db individually. Consider using the Default wrapper
/// instead of this if the `Vec` is "small" enough.
impl<'a, T, E, DS> IntoIterator for &'a Vec<T, DS>
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
            vec: self,
        }
    }
}

/// This can be quite slow as it gets each element from
/// the db individually. Consider using the Default wrapper
/// instead of this if the `VecDeque` is "small" enough.
impl<T, E, DS> Vec<T, DS>
where
    E: fmt::Debug,
    T: Serialize + DeserializeOwned,
    DS: DataStore<DbError = E>,
{
    pub fn iter(&self) -> Iter<T, E, DS> {
        Iter {
            current: 0,
            vec: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use crate::wrapper::Vec;

    #[test]
    fn trivial() {
        let vec: Vec<u16, _> = empty();
        vec.push(&42).unwrap();
        vec.push(&13).unwrap();
        vec.push(&7).unwrap();

        let mut sum = 0;
        for elem in &vec {
            sum += elem.unwrap();
        }
        assert_eq!(sum, 62);
    }

    #[test]
    fn push_post_iter() {
        let vec: Vec<u16, _> = empty();
        vec.push(&42).unwrap();
        vec.push(&13).unwrap();

        let iter = vec.into_iter();
        vec.push(&7).unwrap();

        let mut sum = 0;
        for elem in iter {
            sum += elem.unwrap();
        }
        assert_eq!(sum, 62);
    }

    #[test]
    fn pop_post_iter_is_seen() {
        let vec: Vec<u16, _> = empty();
        vec.push(&42).unwrap();
        vec.push(&13).unwrap();

        let mut sum = 0;
        let iter = vec.into_iter();
        vec.pop().unwrap();

        for elem in iter {
            sum += elem.unwrap();
        }
        assert_eq!(sum, 42);
    }

    #[test]
    fn pop_during_iter() {
        let vec: Vec<u16, _> = empty();
        vec.push(&42).unwrap();
        vec.push(&13).unwrap();

        let mut iter = vec.into_iter();
        iter.next();
        iter.next();
        vec.pop().unwrap();
        assert!(iter.next().is_none());
    }
}
