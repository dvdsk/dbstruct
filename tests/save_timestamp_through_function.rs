use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::mem;
use tempdir::TempDir;

#[dbstruct::dbstruct]
struct PersistentData {
    playlist_last_played: HashMap<String, u64>,
}

pub(crate) struct Db {
    database: PersistentData<sled::Tree>,
}

impl Db {
    pub(crate) fn open(path: &Path) -> Self {
        let db = sled::Config::default()
            .path(path)
            .cache_capacity(1_000_000)
            .open()
            .unwrap()
            .open_tree("boom")
            .unwrap();
        Db {
            database: PersistentData::new(db).unwrap(),
        }
    }

    pub(crate) fn now_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub(crate) fn fetch_last_played(&self, playlist: &str) -> Option<u64> {
        self.database
            .playlist_last_played()
            .get(&playlist.to_owned())
            .unwrap()
    }

    pub(crate) fn store_last_played(&self, playlist: &str, last_played: u64) {
        self.database
            .playlist_last_played()
            .set(&playlist.to_owned(), &last_played)
            .unwrap();
    }
}

/// run twice to replicate issue #7
#[test]
fn recover_from_db() {
    // let dir = TempDir::new("dbstruct_test").unwrap();
    // let path = dir.path().join("sled_db");
    let path = Path::new("fixed");
    let db = Db::open(&path);

    let playlist = "test_playlist_name";
    let last_played = Db::now_timestamp();

    db.store_last_played(&playlist, last_played);
    let fetched = db.fetch_last_played(&playlist).unwrap();
    assert_eq!(fetched, last_played);

    // mem::drop(db);
    // let db = Db::open(&path);
    // let fetched = db.fetch_last_played(&playlist).unwrap();
    // assert_eq!(fetched, last_played);
}
