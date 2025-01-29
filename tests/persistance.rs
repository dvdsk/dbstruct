#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
    primes: Vec<u32>,
}

#[test]
fn push_persistance() {
    let dir = tempdir::TempDir::new("dbstruct_push_persistence").unwrap();
    let path = dir.path().join("db");

    let db = Test::new(&path).unwrap();

    db.the_field().set(&8).unwrap();

    db.primes().push(&2).unwrap();
    db.primes().push(&3).unwrap();
    db.primes().push(&5).unwrap();
    db.primes().push(&7).unwrap();

    std::mem::drop(db);
    let db = Test::new(&path).unwrap();
    assert_eq!(8u8, db.the_field().get().unwrap());
    assert_eq!(Some(7), db.primes().pop().unwrap());
    assert_eq!(Some(5), db.primes().pop().unwrap());
}

#[test]
fn clear_persistence() {
    let dir = tempdir::TempDir::new("dbstruct_clear_persistence").unwrap();
    let path = dir.path().join("db");
    let db = Test::new(&path).unwrap();

    db.primes().push(&2).unwrap();
    db.primes().push(&3).unwrap();
    db.primes().push(&5).unwrap();
    db.primes().push(&7).unwrap();
    db.primes().clear().unwrap();

    std::mem::drop(db);
    let db = Test::new(&path).unwrap();
    assert_eq!(None, db.primes().pop().unwrap());
    assert_eq!(0, db.primes().len());

    db.primes().push(&2).unwrap();
    assert_eq!(Some(2), db.primes().pop().unwrap());
}
