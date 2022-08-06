use core::fmt;

#[doc(hidden)]
pub use dbstruct_derive::*;

pub mod stores;
pub mod traits;
pub use traits::DataStore;
pub mod wrappers;

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
