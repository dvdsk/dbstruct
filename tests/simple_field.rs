#[dbstruct::dbstruct(db=sled)]
pub struct SledTest {
    #[dbstruct(Default)]
    the_field: u8,
}

#[test]
fn sled_backend() {
    let dir = tempdir::TempDir::new("dbstruct_tests").unwrap();
    let path = dir.path().join("simple_field_db");

    let db = SledTest::new(path).unwrap();

    db.the_field().set(&8).unwrap();
    assert_eq!(8u8, db.the_field().get().unwrap());
}

#[dbstruct::dbstruct(db=btreemap)]
pub struct BtreeMapTest {
    #[dbstruct(Default)]
    the_field: u8,
}

#[test]
fn btreemap_backend() {
    let db = BtreeMapTest::new().unwrap();

    db.the_field().set(&8).unwrap();
    assert_eq!(8u8, db.the_field().get().unwrap());
}
