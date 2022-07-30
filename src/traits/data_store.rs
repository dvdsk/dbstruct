use core::fmt;

pub trait DataStore<K, V> {
    type Error: fmt::Debug;
    fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    fn remove(&self, key: &K) -> Result<Option<V>, Self::Error>;
    fn insert<'a>(&self, key: &'a K, val: &'a V) -> Result<Option<V>, Self::Error>;
}

pub trait Atomic<K, V>: DataStore<K, V> {
    fn atomic_update(&self, key: &K, op: impl FnMut(V) -> V + Clone) -> Result<(), Self::Error>;
    /// on error the update is aborted
    fn conditional_update(&self, key: &K, new: &V, expected: &V) -> Result<(), Self::Error>;
}
