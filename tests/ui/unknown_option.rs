use dbstruct::dbstruct;

#[dbstruct(starship=enterprise)]
struct Test {
    field: Option<u8>,
}

fn main() {}
