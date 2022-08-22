#[dbstruct::dbstruct]
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
	let ds = dbstruct::stores::HashMap::default();
	let db = Test::new(ds).unwrap();

	db.the_awnser().set(&42).unwrap();
	assert_eq!(42u8, db.the_awnser().get().unwrap());

	db.primes().push(2).unwrap();
	db.primes().push(3).unwrap();
	db.primes().push(5).unwrap();
	db.primes().push(7).unwrap();
	assert_eq!(Some(7), db.primes().pop().unwrap());

	assert_eq!(String::from("42"), db.the_result().get().unwrap());
}


// use std::clone::Clone;
// use std::sync::Arc;
// use std::sync::atomic::AtomicUsize;
//
// use dbstruct::{DataStore, Error, wrappers, stores};
// use dbstruct::traits::data_store::Orderd;
//
// pub struct Test<DS: DataStore + Orderd + Clone>
// {
//     ds: DS,
//     primes_len: Arc<AtomicUsize>,
// }
// impl<DS> Test<DS>
// where
//     DS: DataStore + Orderd + Clone,
// {
//     pub fn new(ds: DS) -> Result<Self, Error<DS::Error>> {
//         let primes = Orderd::get_lt(&ds, &1u8)?
//             .map(|(len, _): (u8, u32)| len)
//             .unwrap_or(0);
//         Ok(Self {
//             ds,
//             primes_len: Arc::new(AtomicUsize::new(0)),
//         })
//     }
//     fn the_awnser(&self) -> wrappers::DefaultTrait<u8, DS> {
//         wrappers::DefaultTrait::new(self.ds.clone(), 1u8)
//     }
//     fn primes(&self) -> wrappers::Vec<u32, DS> {
//         wrappers::Vec::new(self.ds.clone(), 0u8, self.primes_len.clone())
//     }
//     fn the_result(&self) -> wrappers::DefaultValue<String, DS> {
//         let default_value = format!("{}", 20+2+20);
//         wrappers::DefaultValue::new(self.ds.clone(), 2u8, default_value)
//     }
// }
// fn main() {
//     let ds = stores::HashMap::default();
//     let db = Test::new(ds).unwrap();
//     db.the_awnser().set(&42).unwrap();
// }
