use std::ops::Range;

#[dbstruct::dbstruct(db=btreemap)]
pub struct Test {
    range: HashMap<Range<u32>, u32>,
    to_digit: HashMap<String, u32>,
    plus10: HashMap<u8, u8>,
}

#[test]
fn iterator_visits_all_elements() {
    let test = Test::new().unwrap();
    test.plus10().insert(&1, &11).unwrap();
    test.plus10().insert(&2, &12).unwrap();
    test.plus10().insert(&3, &13).unwrap();

    let collected: Vec<(u8, u8)> = test.plus10().iter().map(Result::unwrap).collect();
    assert!(collected.contains(&(1, 11)));
    assert!(collected.contains(&(2, 12)));
    assert!(collected.contains(&(3, 13)));
}

#[test]
fn update_str_key() {
    let test = Test::new().unwrap();
    for _ in 0..10 {
        let curr = test.to_digit().get("counter").unwrap().unwrap_or(0);
        test.to_digit().insert("counter", &(curr + 1)).unwrap();
    }

    assert_eq!(test.to_digit().get("counter").unwrap(), Some(10));
}

#[test]
fn update_rng_key() {
    let test = Test::new().unwrap();
    let key = 10..12;

    for _ in 0..10 {
        let curr = test.range().get(&key).unwrap().unwrap_or(0);
        test.range().insert(&key, &(curr + 1)).unwrap();
    }

    assert_eq!(test.range().get(&key).unwrap(), Some(10));
}
