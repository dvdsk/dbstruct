use std::mem;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tempdir::TempDir;

mod setup_tracing;

#[dbstruct::dbstruct]
struct Db {
    map: HashMap<String, u64>,
}

/// run twice to replicate issue #7
#[test]
fn recover_from_db() {
    setup_tracing::setup("");

    // let dir = TempDir::new("dbstruct_test").unwrap();
    // let path = dir.path().join("sled_db");
    let path = Path::new("fixed");

    let db = sled::Config::default()
        .path(path)
        .cache_capacity(1_000_000)
        .open()
        .unwrap()
        .open_tree("boom")
        .unwrap();
    let database = Db::new(db).unwrap();

    let test_value = "test_playlist_name";
    let test_key = 235897u64; //Db::now_timestamp();
                              //
    database
        .map()
        .set(&test_value.to_owned(), &test_key)
        .unwrap();

    let fetched = database.map().get(&test_value.to_owned()).unwrap().unwrap();
    assert_eq!(fetched, test_key);

    // mem::drop(db);
    // let db = Db::open(&path);
    // let fetched = db.fetch_last_played(&playlist).unwrap();
    // assert_eq!(fetched, last_played);
}
