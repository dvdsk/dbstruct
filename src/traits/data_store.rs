use core::fmt;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait DataStore<K, V>
where
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
    type Error: fmt::Debug;
    fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    fn remove<'a>(&self, key: &'a K) -> Result<Option<V>, Self::Error>;
    fn insert<'a>(&self, key: &'a K, val: &'a V) -> Result<Option<V>, Self::Error>;
}

pub trait Atomic<K, V>: DataStore<K, V>
where
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
    fn atomic_update(&self, key: &K, op: impl FnMut(V) -> V + Clone) -> Result<(), Self::Error>;
    /// on error the update is aborted
    fn conditional_update(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error>;

}
