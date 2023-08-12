//! Create a typed embedded database by defining a struct. Interact with the database through getters and setters. Choose how values missing in the database are represented. Standard library types `Vec`, `HashMap` and `Option` have special getters and setters to mimic their standard library functionality. You can push and pop from vecs.
//!
//! Choose out of various popular key-value databases then instantiate the struct providing only the db path. Alternatively pass any object that implements `dbstruct::DataStore`.
//!
//!
//! ## How to use `derive(dbstruct)`
//! Lets go through an example, there are many more [here](https://github.com/dvdsk/dbstruct/tree/main/examples):
//!
//!```rust
//!use std::path::Path;
//!
//!#[dbstruct::dbstruct(db=sled)]
//!pub struct Test {
//!    #[dbstruct(Default)]
//!    the_awnser: u8,
//!    primes: Vec<u32>,
//!    #[dbstruct(Default="format!(\"{}\", 20+2+20)")]
//!    the_result: String,
//!}
//!
//!fn main() {
//!    // a wrapper around a HashMap that implements the
//!    // `DataStore` trait
//!    let db = Test::new(&Path::new("the_db2")).unwrap();
//!
//!    db.the_awnser().set(&42).unwrap();
//!	   assert_eq!(42u8, db.the_awnser().get().unwrap());
//!
//!    db.primes().push(2).unwrap();
//!    db.primes().push(3).unwrap();
//!    db.primes().push(5).unwrap();
//!	   db.primes().push(7).unwrap();
//!	   assert_eq!(Some(7), db.primes().pop().unwrap());
//!
//!	   assert_eq!(String::from("42"), db.the_result().get().unwrap());
//!}
//!```
//!
//! Here `derive(dbstruct)` instructs Rust to transform the input struct to a typed database. Every
//! field is replaced with a *method* with the *same name* that returns a [`wrapper`]. The various attributes used to
//! customise the generated method.
//!
//! First, define a struct, whatever its name. This will become your database object. Its fields
//! will become keys or prefixes in the database. Now add the dbstruct attribute and choose a
//! database using `db=<your chosen database>`. Set `db=trait` to use any object you have implemented
//! dbstructs [traits][traits::data_store] for. Finally determine how to deal with missing values. You can
//! use the types [`Default`] implementation, generate missing values from an expression or wrap your type in
//! [`Option`].
//!
//! ## Supported databases
//!
//!| Name                                    | advantage | attribute option |
//!|-----------------------------------------|-----------|------------------|
//!| [Sled](https://crates.io/crates/sled)   | pure Rust | `db=sled`        |
//!
//! ## How it works
//! dbstruct replaces the *fields* in your struct *with methods*. Each method returns a [`wrapper`]
//! that allows getting and setting values. While your program runs the fields of a struct are
//! stored in memory, there values lost the program stops. The wrapper store the values in a
//! database.
//!
//! ##### Missing values
//! When dbstruct can not find a field in the database it needs to know what to return. You must
//! tell dbstruct how to treat missing values. The simplest way is to wrap your type in an
//! [`Option`]. Then dbstruct will return None if the value is missing. Alternatively you can instruct dbstruct
//! to use the types [`Default`] implementation or set an expression to generate a default value.
//!
//! ##### Special wrapper
//! Some fields get methods that return special wrapper. These wrappers mimic the fields type and
//! handle missing values on their own. Struct fields with type Vec are transformed into methods that
//! return a Vec wrapper. It allows pushing and popping values. You can opt out of this by defining
//! how to handle missing values (see above)
//!
//! See [`wrapper`] for a complete list.

use core::fmt;

#[doc(hidden)]
pub use dbstruct_derive::*;

pub mod stores;
pub mod traits;
pub use traits::{ByteStore, DataStore};
pub mod wrapper;

pub use sled;

/// An Error type encapulating various issues that may come up during database operation or
/// (de)serializing
#[derive(Debug, thiserror::Error)]
pub enum Error<DbError: fmt::Debug> {
    #[error("value could not be deserialized using bincode")]
    DeSerializingVal(bincode::Error),
    #[error("key could not be deserialized using bincode")]
    DeSerializingKey(bincode::Error),
    #[error("value could not be serialized using bincode")]
    SerializingValue(bincode::Error),
    #[error("could not serialize key using bincode")]
    SerializingKey(bincode::Error),
    #[error("the database returned an error")]
    Database(#[from] DbError),
}

#[doc = include_str!("../Readme.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
