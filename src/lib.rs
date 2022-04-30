#[doc(hidden)]
pub use structdb_derive::*;
pub use sled;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database operation failed")]
    Sled(#[from] sled::Error),
    #[error("failed to serialize type")]
    Serializing(bincode::Error),
    #[error("error deserializing, did the type of this field change?")]
    DeSerializing(bincode::Error),
}
