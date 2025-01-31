# dbstruct

> **Derive a database from a struct**

[![Crates.io](https://img.shields.io/crates/v/dbstruct?style=flat-square)](https://crates.io/crates/dbstruct)
[![Crates.io](https://img.shields.io/crates/d/dbstruct?style=flat-square)](https://crates.io/crates/dbstruct)
[![API](https://docs.rs/dbstruct/badge.svg)](https://docs.rs/dbstruct)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)
**This is an early release, the API is mostly stable but might still change**

Create a typed embedded database by defining a struct. Interact with the database through getters and setters. Choose how values missing in the database are represented. Standard library types `Vec`, `HashMap` and `Option` have special getters and setters to mimic their standard library functionality. You can push and pop from vecs. 

Choose out of various popular key-value databases then instantiate the struct providing only the db path. Alternatively pass any object that implements `dbstruct::DataStore`. 


## Use case
dbstruct is ideal when:
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

	db.primes().push(&2).unwrap();
	db.primes().push(&3).unwrap();
	db.primes().push(&5).unwrap();
    db.primes().push(&7).unwrap();
    assert_eq!(Some(7), db.primes().pop().unwrap());

    assert_eq!(String::from("42"), db.the_result().get().unwrap());
}
```

## Out of the box support
| Name                                    | advantage | attribute option |
|-----------------------------------------|-----------|------------------|
| [Sled](https://crates.io/crates/sled)   | pure Rust | `db=sled`        |
| BTreeMap, does not store anything!      | testing   | `db=btreemap`    |

work in progress: rocksdb

## Future Work
These are some features I am planning to work on, in no particular order. If you miss anything *please let me know* via an issue!
- Example workflow for migrations.
- (Dis)Allow access from multiple threads cloning the struct
- Flushing the database, explicitly via a function on the struct and implicitly whenever a field changes. Will be configurable through an attribute on the struct and a field specifically.
- Expand the wrapper API to more closely match that of their standard library counterparts.
- Async support for flushing the database.
- Figure out how to represent transactions _(hard if even possible)_

## Similar Crates
- [SQLx](https://crates.io/crates/sqlx)
- [cornucopia](https://crates.io/crates/cornucopia) Generate type-checked Rust from your PostgreSQL.
- [losfair/RefineDB](https://github.com/losfair/RefineDB) A strongly-typed document database that runs on any transactional key-value store
- [chronicl/typed-sled](https://crates.io/crates/typed-sled) builds on top of sled and offers an API that is similar to a `BTreeMap<K, V>`
- [sea-orm](https://crates.io/crates/sea-orm) a relational ORM to help you build web services in Rust with the familiarity of dynamic languages
- [native_db](https://github.com/vincent-herlemont/native_db) embedded database for multi-platform apps. Sync Rust types effortlessly.
