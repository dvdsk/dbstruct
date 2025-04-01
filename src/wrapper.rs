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

mod option;
mod default_val;
mod default_trait;
mod vec;
mod vec_deque;
mod map;

pub use option::OptionValue;
pub use default_val::DefaultValue;
pub use default_trait::DefaultTrait;
pub use map::Map;
pub use vec_deque::VecDeque;
pub use vec::Vec;

// we need to expose prefixed for the generated 
// code to be able to access it
#[doc(hidden)]
pub use vec::Prefixed as VecPrefixed;
#[doc(hidden)]
pub use vec_deque::Prefixed as DequePrefixed;
