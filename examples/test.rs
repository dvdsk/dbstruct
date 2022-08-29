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
	let db = Test::new(&Path::new("hi")).unwrap();

	db.the_awnser().set(&42).unwrap();
	assert_eq!(42u8, db.the_awnser().get().unwrap());

	db.primes().push(2).unwrap();
	db.primes().push(3).unwrap();
	db.primes().push(5).unwrap();
	db.primes().push(7).unwrap();
	assert_eq!(Some(7), db.primes().pop().unwrap());

	assert_eq!(String::from("42"), db.the_result().get().unwrap());
}
