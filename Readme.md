**This is an early release, the API is unstable and might still change in**

Create a typed database by defining a struct. *dbstruct* builds on top of anything that implements `dbstruct::DataStore` or `dbstruct::BytesStore` and provides a typed API similar to the standard library. An implementation for [sled](https://crates.io/crates/sled) is provided. 

## Usecase
*dbstruct* is ideal when:
- writing a simple app that needs some fast data storage.
- quickly getting a storage layer done when prototyping a system that you can later replace.

## Example
```rust
#[dbstruct::dbstruct]
pub struct Test {
    #[dbstruct(Default)]
    the_awnser: u8,
	primes: Vec<u32>,
	// #[dbstruct(Default="format!(\"{}\", 20+2+20)")]
	// the_result: String,
}

fn main() {
	// a wrapper around a HashMap that implements the 
	// `DataStore` trait
    let ds = dbstruct::stores::HashMap::default();
    let db = Test::new(ds).unwrap();

    db.the_awnser().set(&42).unwrap();
    assert_eq!(42u8, db.the_awnser().get().unwrap());

	// db.primes().push(2).unwrap();
	// db.primes().push(3).unwrap();
	// db.primes().push(5).unwrap();
	// db.primes().push(7).unwrap();
	// assert_eq!(7, db.primes().pop().unwrap());

	// assert_eq!(String::from("42"), db.the_result().get().unwrap());
}
```
