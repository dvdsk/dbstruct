//! provides a non persistent store for testing

#[cfg(feature = "sled")]
mod sled;
#[cfg(feature = "rocksdb")]
mod rocksdb;

mod hashmap;
mod btreemap;

#[deprecated(
    since = "0.3.0",
    note = "Use BTreeMap test backend instead"
)]
pub use hashmap::HashMap;
pub use hashmap::Error as HashMapError;
pub use btreemap::BTreeMap;
pub use btreemap::Error as BTreeMapError;
// interesting discussion about key value db alternatives to sled: 
// https://gitlab.com/famedly/conduit/-/issues/74
// one interesting one is heed (wraps LMDB)
