use crate::Error;
use core::marker::PhantomData;
use std::fmt;

use crate::traits::byte_store;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::Map;

pub struct Iter<'a, K, V, E, DS>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: byte_store::Ordered<DbError = E>,
{
    prefix: u8,
    prev_key_bytes: Vec<u8>,
    phantom_val: PhantomData<V>,
    phantom_key: PhantomData<K>,
    ds: &'a DS,
}

impl<K, V, E, DS> Iterator for Iter<'_, K, V, E, DS>
where
    E: fmt::Debug,
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    DS: byte_store::Ordered<DbError = E>,
{
    type Item = Result<(K, V), Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (key, val) = match byte_store::Ordered::get_gt(self.ds, &self.prev_key_bytes) {
            Ok(Some((key, val))) => (key, val),
            Ok(None) => return None,
            Err(e) => return Some(Err(Error::Database(e))),
        };

        let key = key.as_ref();
        if key[0] != self.prefix {
            return None;
        }

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
    DS: byte_store::Ordered<DbError = E>;

impl<K, V, E, DS> Iterator for Values<'_, K, V, E, DS>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: byte_store::Ordered<DbError = E>,
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
    DS: byte_store::Ordered<DbError = E>;

impl<K, V, E, DS> Iterator for Keys<'_, K, V, E, DS>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    E: fmt::Debug,
    DS: byte_store::Ordered<DbError = E>,
{
    type Item = Result<K, Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(|(key, _)| key))
    }
}

impl<Key, Value, E, DS> Map<'_, Key, Value, DS>
where
    E: fmt::Debug,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
    DS: byte_store::Ordered<DbError = E>,
{
    /// An iterator visiting all key-value pairs in fixed though arbitrary order. The
    /// order depending on the underlying database implementation. The iterator element
    /// type is `Result<(Key, Value), dbstruct::Error<E>>`.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().insert(&1, &"a".to_owned())?;
    /// db.map().insert(&2, &"b".to_owned())?;
    /// db.map().insert(&3, &"c".to_owned())?;
    ///
    /// for res in db.map().iter() {
    ///     let (key, val) = res?;
    ///     println!("key: {key} val: {val}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> Iter<Key, Value, E, DS> {
        Iter {
            prefix: self.prefix,
            prev_key_bytes: vec![self.prefix],
            phantom_val: PhantomData,
            phantom_key: PhantomData,
            ds: &self.tree,
        }
    }

    /// An iterator visiting all key in fixed though arbitrary order. The
    /// order depending on the underlying database implementation. The iterator element
    /// type is `Result<Key, dbstruct::Error<E>>`.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().insert(&1, &"a".to_owned())?;
    /// db.map().insert(&2, &"b".to_owned())?;
    /// db.map().insert(&3, &"c".to_owned())?;
    ///
    /// for key in db.map().keys() {
    ///     let key = key?;
    ///     println!("key: {key}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn values(&self) -> Values<Key, Value, E, DS> {
        Values(Iter {
            prefix: self.prefix,
            prev_key_bytes: vec![self.prefix],
            phantom_val: PhantomData,
            phantom_key: PhantomData,
            ds: &self.tree,
        })
    }

    /// An iterator visiting all values in fixed though arbitrary order. The
    /// order depending on the underlying database implementation. The iterator element
    /// type is `Result<Value, dbstruct::Error<E>>`.
    ///
    /// # Errors
    /// This can fail if the underlying database ran into a problem
    /// or if serialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// #[dbstruct::dbstruct(db=btreemap)]
    /// struct Test {
    ///	    map: HashMap<u16, String>,
    ///	}
    ///
    ///	# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Test::new()?;
    /// db.map().insert(&1, &"a".to_owned())?;
    /// db.map().insert(&2, &"b".to_owned())?;
    /// db.map().insert(&3, &"c".to_owned())?;
    ///
    /// for val in db.map().values() {
    ///     let val = val?;
    ///     println!("val: {val}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn keys(&self) -> Keys<Key, Value, E, DS> {
        Keys(Iter {
            prefix: self.prefix,
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
        assert!(dbg!(&pairs).contains(&(1, 11)));
        assert!(pairs.contains(&(2, 12)));
        assert!(pairs.contains(&(3, 13)));
    }
}
