//! provides a non peristant store for testing

#[cfg(feature = "sled")]
mod sled;
#[cfg(feature = "rocksdb")]
mod rocksdb;

mod hashmap;
mod btreemap;

#[deprecated(
    since = "0.3",
    note = "Use BTreeMap test backend instead"
)]
pub use hashmap::HashMap;
pub use hashmap::Error as HashMapError;
pub use btreemap::BTreeMap;
pub use btreemap::Error as BTreeMapError;
// intresting discussion about key value db alternatives to sled: 
// https://gitlab.com/famedly/conduit/-/issues/74
// one intresting one is heed (wraps LMDB)
