use std::mem;
use tempdir::TempDir;

mod setup_tracing;

#[dbstruct::dbstruct]
struct Db {
    map: HashMap<String, u8>,
}

#[test]
fn re_open() {
    setup_tracing::setup("");

    let dir = TempDir::new("dbstruct_test").unwrap();
    let path = dir.path().join("sled_db");

    let db = sled::Config::default()
        .path(&path)
        .cache_capacity(1_000_000)
        .open()
        .unwrap()
        .open_tree("boom")
        .unwrap();
    let database = Db::new(db).unwrap();

    let test_value = "t";
    let test_key = 42u8;

    database
        .map()
        .set(&test_value.to_owned(), &test_key)
        .unwrap();

    let fetched = database.map().get(&test_value.to_owned()).unwrap().unwrap();
    assert_eq!(fetched, test_key);
    mem::drop(database);

    let db = sled::Config::default()
        .path(&path)
        .cache_capacity(1_000_000)
        .open()
        .unwrap()
        .open_tree("boom")
        .unwrap();
    let database = Db::new(db).unwrap();
    let fetched = database.map().get(&test_value.to_owned()).unwrap().unwrap();
    assert_eq!(fetched, test_key);
}
