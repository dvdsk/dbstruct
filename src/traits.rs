use core::fmt;

pub mod byte_store;
pub mod data_store;

pub use byte_store::ByteStore;
pub use data_store::DataStore;

#[derive(Debug, thiserror::Error)]
pub enum Error<DbError: fmt::Debug> {
    #[error("value could not be deserialized using bincode")]
    DeSerializing(bincode::Error),
    #[error("value could not be serialized using bincode")]
    SerializingValue(bincode::Error),
    #[error("could not serialize key using bincode")]
    SerializingKey(bincode::Error),
    #[error("the database returned an error")]
    Database(#[from] DbError),
}

