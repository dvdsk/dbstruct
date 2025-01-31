//! You can implement some of these traits to use an unsupported database

pub mod byte_store;
pub mod data_store;

use core::fmt;

pub use byte_store::ByteStore;
pub use data_store::DataStore;

/// Error returned by [`TryExtend::try_extend`].
#[derive(Debug, thiserror::Error)]
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

pub trait TryExtend<T> {
    type DbError: fmt::Debug;

    /// Fallible version of the std's
    /// [`Extend`](https://doc.rust-lang.org/std/iter/trait.Extend.html) trait.
    /// Stops on the first error encountered and returns the iterator and the
    /// item we failed to insert.
    #[allow(clippy::type_complexity)]
    fn try_extend<I>(
        &mut self,
        iter: I,
    ) -> Result<(), ExtendError<T, I::IntoIter, crate::Error<Self::DbError>>>
    where
        I: IntoIterator<Item = T>;
}
