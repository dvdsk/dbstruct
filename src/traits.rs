//! You can implement some of these traits to use an unsupported database

pub mod byte_store;
pub mod data_store;

pub use byte_store::ByteStore;
pub use data_store::DataStore;
