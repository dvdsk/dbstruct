use dbstruct::dbstruct;

#[dbstruct(db=sled, db=trait)]
struct Test {
    field: Option<u8>,
}

fn main() {}
