#[cfg(feature = "sled")]
mod sled;
#[cfg(feature = "rocksdb")]
mod rocksdb;

mod hashmap;
pub use hashmap::HashMap;
pub use hashmap::Error as HashMapError;
// intresting discussion about key value db alternatives to sled: 
// https://gitlab.com/famedly/conduit/-/issues/74
// one intresting one is heed (wraps LMDB)
