# dbstruct

> **Derive a database from a struct**

[![Crates.io](https://img.shields.io/crates/v/dbstruct?style=flat-square)](https://crates.io/crates/dbstruct)
[![Crates.io](https://img.shields.io/crates/d/dbstruct?style=flat-square)](https://crates.io/crates/dbstruct)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)
**This is an early release, the API is mostly stable but might still change**

Create a typed embedded database by defining a struct. Interact with the database through getters and setters. Choose how values missing in the database are represented. Standard library types `Vec`, `HashMap` and `Option` have special getters and setters to mimic their standard library functionality. You can push and pop from vecs. 

Choose out of various popular key-value databases then instantiate the struct providing only the db path. Alternatively pass any object that implements `dbstruct::DataStore`. 


## Usecase
*dbstruct* is ideal when:
- Writing a simple app that needs some form of persistence.
- Quickly getting a storage layer done when developing a system that you can later replace.


## Example
```rust
use std::path::Path;

#[dbstruct::dbstruct(db=sled)]
pub struct Test {
	#[dbstruct(Default)]
	the_awnser: u8,
	primes: Vec<u32>,
	#[dbstruct(Default="format!(\"{}\", 20+2+20)")]
	the_result: String,
}

fn main() {
	// a wrapper around a HashMap that implements the 
	// `DataStore` trait
	let db = Test::new(&Path::new("the_db")).unwrap();

	db.the_awnser().set(&42).unwrap();
	assert_eq!(42u8, db.the_awnser().get().unwrap());

	db.primes().push(2).unwrap();
	db.primes().push(3).unwrap();
	db.primes().push(5).unwrap();
	db.primes().push(7).unwrap();
	assert_eq!(Some(7), db.primes().pop().unwrap());

	assert_eq!(String::from("42"), db.the_result().get().unwrap());
}
```

## Out of the box support
| Name                                    | advantage | attribute option |
|-----------------------------------------|-----------|------------------|
| [Sled](https://crates.io/crates/sled)   | pure Rust | `db=sled`        |

work in progress: rocksdb

## Future Work
These are some features I am planning to work on, in no particular order.
- Example workflow for migrations.
- (Dis)Allow access from multiple threads cloning the struct
- Flushing the database, explicitly via a function on the struct and implicitly whenever a field changes. Will be configurable through an attribute on the struct and a field specifically.
- Expand the wrappers API to more closely match that of their standard library counterparts.
- Async support for flushing the database.
- Figure out how to represent transactions _(hard if even possible)_
