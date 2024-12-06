#[dbstruct::dbstruct(db=sled)]
#[derive(Clone)]
pub struct Test {
    computers: HashMap<String, usize>,
}

fn main() {
    let dir = tempdir::TempDir::new("dbstruct_examples").unwrap();
    let path = dir.path().join("map_wrapper");

    let db = Test::new(&path).unwrap();
    db.computers().insert(&"Deep Thought".to_owned(), &42).unwrap();
    db.computers().remove(&"Deep Thought".to_owned()).unwrap();
}
