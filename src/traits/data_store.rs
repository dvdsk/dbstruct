//! The traits used by the wrapper to operate on the database.
use core::fmt;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Base trait needed by every wrapper. It is usually more convenient to implement
/// [`ByteStore`][super::byte_store::ByteStore] instead.
pub trait DataStore {
    type Error: fmt::Debug;
    fn get<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        K: Serialize,
        V: DeserializeOwned;
    fn remove<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        K: Serialize,
        V: DeserializeOwned;
    fn insert<'a, K, V>(&self, key: &'a K, val: &'a V) -> Result<Option<V>, Self::Error>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned;
}

/// This trait enables wrapper to provide `update` and `conditional` update. 
/// It is usually more convenient to implement
/// [`byte_store::Atomic`][super::byte_store::Atomic] instead.
pub trait Atomic: DataStore {
    fn atomic_update<K, V>(
        &self,
        key: &K,
        op: impl FnMut(V) -> V + Clone,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned;
    /// on error the update is aborted
    fn conditional_update<K, V>(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned;
}

/// This trait needed for the Vec wrapper. It is usually more convenient to implement
/// [`byte_store::Ordered`][super::byte_store::Ordered] instead.
pub trait Ordered: DataStore {
    fn get_lt<K, V>(&self, key: &K) -> Result<Option<(K, V)>, Self::Error>
    where
        K: Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned;
}
