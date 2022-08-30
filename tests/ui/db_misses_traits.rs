use dbstruct::dbstruct;

#[dbstruct(db=hashmap)]
struct Test {
    field: Vec<u8>,
}

fn main() {}
