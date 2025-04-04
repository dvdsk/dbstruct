//! Wrapper are how *dbstruct* reads and writes to the database. They handle
//! serializing and deserializing, provide the API and determine how to handle
//! missing data.
//!
//! There are two types of wrapper:
//! - Those describing how to handle missing values. These at the minimum offer
//!   you `get` and `set`. Depending on the [`traits`][crate::traits::data_store] the
//!   database you chose implements they may also support `update` and
//!   `conditional_update`.
//! - Wrapper that mimic the API of a standard library type.

mod default_trait;
mod default_val;
pub mod map;
mod option;
mod vec;
mod vec_deque;

pub use default_trait::DefaultTrait;
pub use default_val::DefaultValue;
pub use map::Map;
pub use option::OptionValue;
pub use vec::Vec;
pub use vec_deque::VecDeque;

// We need to expose prefixed for the generated
// code to be able to access it
#[doc(hidden)]
pub use vec::Prefixed as VecPrefixed;
#[doc(hidden)]
pub use vec_deque::Prefixed as DequePrefixed;

/// Not all wrappers should be sync, negative marker traits are unstable
/// instead we use this.
pub(crate) type PhantomUnsync = std::marker::PhantomData<std::cell::Cell<()>>;
