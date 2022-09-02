#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}

#[test]
fn main() {
    let dir = tempdir::TempDir::new("dbstruct_tests").unwrap();
    let path = dir.path().join("simple_field_db");

    let db = Test::new(path).unwrap();

    db.the_field().set(&8).unwrap();
    assert_eq!(8u8, db.the_field().get().unwrap());
}
