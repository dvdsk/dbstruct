//! provides a non persistent store for testing

#[cfg(feature = "rocksdb")]
mod rocksdb;
#[cfg(feature = "sled")]
mod sled;

mod btreemap;
mod hashmap;

pub use btreemap::BTreeMap;
pub use btreemap::Error as BTreeMapError;
pub use hashmap::Error as HashMapError;
#[deprecated(since = "0.3.0", note = "Use BTreeMap test backend instead")]
pub use hashmap::HashMap;
// interesting discussion about key value db alternatives to sled:
// https://gitlab.com/famedly/conduit/-/issues/74
// one interesting one is heed (wraps LMDB)
