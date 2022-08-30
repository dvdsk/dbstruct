use std::path::Path;

#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}

#[test]
fn main() {
    let db = Test::new(&Path::new("test")).unwrap();

    db.the_field().set(&8).unwrap();
    assert_eq!(8u8, db.the_field().get().unwrap());
}
