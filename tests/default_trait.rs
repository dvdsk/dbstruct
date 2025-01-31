use std::collections::HashMap;

#[dbstruct::dbstruct(db=trait)]
pub struct Test {
    /// a small list that we want structdb to store as a single db entry 
    #[dbstruct(Default)]
    small_list: Vec<u8>,
    /// a small map that we want structdb to store as a single db entry 
    #[dbstruct(Default)]
    small_map: HashMap<usize, u32>,
}

#[test]
fn main() {
    let ds = dbstruct::stores::HashMap::default();
    let db = Test::new(ds).unwrap();

    assert!(db.small_list().get().unwrap().is_empty());
    assert!(db.small_map().get().unwrap().is_empty());

    let list = vec![1,2,3,4];
    db.small_list().set(&list).unwrap();
    assert_eq!(list, db.small_list().get().unwrap());

    let map: HashMap<_,_> = (4..8).enumerate().collect();
    db.small_map().set(&map).unwrap();
    assert_eq!(map, db.small_map().get().unwrap());
}
