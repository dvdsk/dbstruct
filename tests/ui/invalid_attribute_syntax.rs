use dbstruct::dbstruct;

#[dbstruct("db"=sled)]
struct Test {
    field: Option<u8>,
}

fn main() {}
