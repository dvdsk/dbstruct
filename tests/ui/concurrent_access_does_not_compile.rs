use dbstruct::dbstruct;

#[dbstruct(db=sled)]
struct Test {
    list: Vec<u8>,
}

// SHOULD NOT COMPILE
fn main() {
    let test = Test::new("").unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            test.list().push(&5).unwrap();
        });
        s.spawn(|| {
            test.list().push(&6).unwrap();
        });
    })
}
