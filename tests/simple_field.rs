#[dbstruct::dbstruct]
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}

fn main() {
    let ds = dbstruct::stores::HashMap::default();
    let db = Test::new(ds).unwrap();

    db.the_field().set(&8).unwrap();
    assert_eq!(8u8, db.the_field().get().unwrap());
}
