use crate::Error;
use core::marker::PhantomData;
use std::fmt;

use crate::traits::{byte_store, DataStore};

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::Map;

pub struct Iter<'a, K, V, E, DS>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: DataStore<Error = E> + byte_store::Ordered,
{
    prev_key_bytes: Vec<u8>,
    phantom_val: PhantomData<V>,
    phantom_key: PhantomData<K>,
    ds: &'a DS,
}

impl<'a, K, V, E, DS> Iterator for Iter<'a, K, V, E, DS>
where
    E: fmt::Debug,
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    DS: DataStore<Error = E> + byte_store::Ordered,
{
    type Item = Result<(K, V), Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some((key, val)) = byte_store::Ordered::get_gt(self.ds, &self.prev_key_bytes).unwrap() else {
            return None;
        };

        let key = key.as_ref();
        self.prev_key_bytes.clear();
        self.prev_key_bytes.extend_from_slice(key);

        let key = &key[1..]; // strip prefix
        let key = match bincode::deserialize(key).map_err(Error::DeSerializingKey) {
            Ok(key) => key,
            Err(e) => return Some(Err(e)),
        };
        let val = match bincode::deserialize(val.as_ref()).map_err(Error::DeSerializingVal) {
            Ok(val) => val,
            Err(e) => return Some(Err(e)),
        };
        Some(Ok((key, val)))
    }
}

pub struct Values<'a, K, V, E, DS>(Iter<'a, K, V, E, DS>)
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: DataStore<Error = E> + byte_store::Ordered;

impl<'a, K, V, E, DS> Iterator for Values<'a, K, V, E, DS>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: DataStore<Error = E> + byte_store::Ordered,
{
    type Item = Result<V, Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(|(_, val)| val))
    }
}

pub struct Keys<'a, K, V, E, DS>(Iter<'a, K, V, E, DS>)
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: DataStore<Error = E> + byte_store::Ordered;

impl<'a, K, V, E, DS> Iterator for Keys<'a, K, V, E, DS>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: DataStore<Error = E> + byte_store::Ordered,
{
    type Item = Result<K, Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(|(key, _)| key))
    }
}

impl<'a, Key, Value, E, DS> Map<'a, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: DataStore<Error = E> + byte_store::Ordered,
{
    pub fn iter(&self) -> Iter<Key, Value, E, DS> {
        Iter {
            prev_key_bytes: vec![self.prefix],
            phantom_val: PhantomData,
            phantom_key: PhantomData,
            ds: &self.tree,
        }
    }

    pub fn values(&self) -> Values<Key, Value, E, DS> {
        Values(Iter {
            prev_key_bytes: vec![self.prefix],
            phantom_val: PhantomData,
            phantom_key: PhantomData,
            ds: &self.tree,
        })
    }

    pub fn keys(&self) -> Keys<Key, Value, E, DS> {
        Keys(Iter {
            prev_key_bytes: vec![self.prefix],
            phantom_val: PhantomData,
            phantom_key: PhantomData,
            ds: &self.tree,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn iterator_visits_all_elements() {
        use super::super::tests::*;
        let map = empty();
        map.insert(&1, &11).unwrap();
        map.insert(&2, &12).unwrap();
        map.insert(&3, &13).unwrap();

        let pairs: Vec<(u8, u8)> = map.iter().map(Result::unwrap).collect();
        assert!(dbg!(&pairs).contains(&(1,11)));
        assert!(pairs.contains(&(2,12)));
        assert!(pairs.contains(&(3,13)));
    }
}
