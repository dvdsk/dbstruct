#[dbstruct::dbstruct(db=sled)]
#[derive(Clone)]
pub struct Test {
    computers: HashMap<String, usize>,
}

fn main() {
    let dir = tempdir::TempDir::new("dbstruct_examples").unwrap();
    let path = dir.path().join("map_wrapper");

    let db = Test::new(&path).unwrap();
    db.computers()
        .insert(&"Deep Thought".to_owned(), &42)
        .unwrap();
    db.computers()
        .insert(&"Colossus".to_owned(), &1944)
        .unwrap();
    db.computers().insert(&"ENIAC".to_owned(), &1946).unwrap();
    db.computers()
        .insert(&"System/360".to_owned(), &1953)
        .unwrap();
    db.computers()
        .insert(&"DEC PDP-8".to_owned(), &1970)
        .unwrap();
    db.computers().insert(&"IBM PC".to_owned(), &1981).unwrap();

    db.computers().remove(&"Deep Thought".to_owned()).unwrap();
    let real_computers: Vec<String> = db
        .computers()
        .keys()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(!real_computers.contains(&"Deep Thought".to_owned()))
}
