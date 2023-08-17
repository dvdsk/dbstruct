//! You can implement some of these traits to use an unsupported database

pub mod byte_store;
pub mod data_store;

pub use byte_store::ByteStore;
pub use data_store::DataStore;

/// Error returned by [`TryExtend::try_extend`].
#[derive(Debug, thiserror::Error)]
pub struct ExtendError<T, I, E> {
    /// The items that we tried to add when the error occured.
    pub unadded: T,

    /// The remaining bit of the iterator, it does not include
    /// unadded
    pub iter: I,

    /// The error that occured while trying to extend.
    #[source]
    pub error: E,
}

pub trait TryExtend<T> {
    type Error;
    /// Failible version of the std's
    /// [`Extend`](https://doc.rust-lang.org/std/iter/trait.Extend.html) trait.
    /// Stops on the first error encounterd and returns the iterator and the
    /// item we failed to insert.
    fn try_extend<I>(&mut self, iter: I) -> Result<(), ExtendError<T, I::IntoIter, Self::Error>>
    where
        I: IntoIterator<Item = T>;
}
