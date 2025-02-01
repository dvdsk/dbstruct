//! You can implement some of these traits to use an unsupported database

pub mod byte_store;
pub mod data_store;

pub use byte_store::ByteStore;
pub use data_store::DataStore;

/// Error returned by [`Vec::extend`](crate::wrapper::Vec::extend) or
/// [`Map::extend`](crate::wrapper::Map::extend). when extending the 
/// collection fails.
#[derive(Debug, thiserror::Error)]
#[error("Could not extend collection")]
pub struct ExtendError<T, I, E> {
    /// The items that we tried to add when the error occurred.
    pub unadded: T,

    /// The remaining bit of the iterator, it does not include
    /// unadded
    pub iter: I,

    /// The error that occurred while trying to extend.
    #[source]
    pub error: E,
}
