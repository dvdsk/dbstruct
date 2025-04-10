#![allow(clippy::type_complexity)]
//! The traits used by the wrapper to operate on the database.
use core::fmt;
use std::ops::RangeBounds;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Base trait needed by every wrapper. It is usually more convenient to implement
/// [`ByteStore`][super::byte_store::ByteStore] instead.
pub trait DataStore {
    type DbError: fmt::Debug;
    fn get<K, V>(&self, key: &K) -> Result<Option<V>, crate::Error<Self::DbError>>
    where
        K: Serialize,
        V: DeserializeOwned;
    fn remove<K, V>(&self, key: &K) -> Result<Option<V>, crate::Error<Self::DbError>>
    where
        K: Serialize,
        V: DeserializeOwned;
    fn insert<'a, K, V, OwnedV>(
        &self,
        key: &'a K,
        val: &'a V,
    ) -> Result<Option<OwnedV>, crate::Error<Self::DbError>>
    where
        K: Serialize,
        V: Serialize + ?Sized,
        OwnedV: std::borrow::Borrow<V> + DeserializeOwned;
}

/// This trait enables wrapper to provide `update` and `conditional` update.
/// It is usually more convenient to implement
/// [`byte_store::Atomic`][super::byte_store::Atomic] instead.
pub trait Atomic: DataStore {
    fn atomic_update<K, V>(
        &self,
        key: &K,
        op: impl FnMut(V) -> V + Clone,
    ) -> Result<(), crate::Error<Self::DbError>>
    where
        K: Serialize,
        V: Serialize + DeserializeOwned;
    /// On error the update is aborted
    fn conditional_update<K, V>(
        &self,
        key: &K,
        new: &V,
        expected: &V,
    ) -> Result<(), crate::Error<Self::DbError>>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized;
}

/// This trait needed for the Vec wrapper. It is usually more convenient to implement
/// [`byte_store::Ordered`][super::byte_store::Ordered] instead.
///
/// You can deserialize to a different key than you serialize too.
/// This is useful when using `get_lt` an `InKey` that borrows data. As
/// you need to deserialize to a type owning all its data.
pub trait Ordered: DataStore {
    /// Retrieve the next key and value from the Tree after the provided key.
    ///
    /// ### Note
    /// The order follows the `Ord` implementation for `Vec<u8>`:
    /// `[] < [0] < [255] < [255, 0] < [255, 255] ...`
    /// To retain the ordering of numerical types use big endian representation
    fn get_lt<InKey, OutKey, Value>(
        &self,
        key: &InKey,
    ) -> Result<Option<(OutKey, Value)>, crate::Error<Self::DbError>>
    where
        InKey: Serialize,
        OutKey: Serialize + DeserializeOwned,
        Value: Serialize + DeserializeOwned;

    /// Retrieve the previous key and value from the Tree before the provided key.
    ///
    /// ### Note
    /// The order follows the `Ord` implementation for `Vec<u8>`:
    /// `[] < [0] < [255] < [255, 0] < [255, 255] ...`
    /// To retain the ordering of numerical types use big endian representation
    fn get_gt<InKey, OutKey, Value>(
        &self,
        key: &InKey,
    ) -> Result<Option<(OutKey, Value)>, crate::Error<Self::DbError>>
    where
        InKey: Serialize,
        OutKey: Serialize + DeserializeOwned,
        Value: Serialize + DeserializeOwned;
}

/// This trait expand the functionality of the Map wrapper. It is usually more
/// convenient to implement [`byte_store::Ranged`][super::byte_store::Ranged]
/// instead.
///
/// You can deserialize to a different key than you serialize too. This is
/// useful when using range an `InKey` that borrows data. As you need to
/// deserialize to a type owning all its data.
pub trait Ranged: DataStore {
    fn range<InKey, OutKey, Value>(
        &self,
        range: impl RangeBounds<InKey>,
    ) -> Result<
        impl Iterator<Item = Result<(OutKey, Value), crate::Error<Self::DbError>>>,
        crate::Error<Self::DbError>,
    >
    where
        InKey: Serialize,
        OutKey: Serialize + DeserializeOwned,
        Value: Serialize + DeserializeOwned;
}
