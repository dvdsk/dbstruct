use serde::{de::DeserializeOwned, Serialize};
pub use sled;
#[doc(hidden)]
pub use dbstruct_derive::*;

mod wrappers;
mod traits;
pub use wrappers::{Vec, DefaultValue, DefaultTrait, OptionValue, Map};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database operation failed")]
    Sled(#[from] sled::Error),
    #[error("failed to serialize type")]
    Serializing(bincode::Error),
    #[error("error deserializing, did the type of this field change?")]
    DeSerializing(bincode::Error),
}

#[derive(thiserror::Error, Debug)]
#[error("compare and swap did not find expected value")]
pub struct CompareAndSwapError<T> {
    pub current: T,
    pub proposed: T,
}

/// # Panics
/// Panics if proposed or current in `sled::CompareAndSwapError` is None
impl<T> TryFrom<sled::CompareAndSwapError> for CompareAndSwapError<T>
where
    T: DeserializeOwned + Serialize,
{
    type Error = Error;
    fn try_from(e: sled::CompareAndSwapError) -> Result<Self, Self::Error> {
        let sled::CompareAndSwapError { current, proposed } = e;
        let current = current.expect("db values should always be set");
        let current = bincode::deserialize(&current).map_err(Error::Serializing)?;
        let proposed = proposed.expect("db values should always be set");
        let proposed = bincode::deserialize(&proposed).map_err(Error::Serializing)?;
        Ok(CompareAndSwapError { current, proposed })
    }
}
