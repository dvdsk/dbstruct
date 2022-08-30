use dbstruct::dbstruct;

#[dbstruct(db=starship_voyager)]
struct Test {
    field: Option<u8>,
}

fn main() {}
