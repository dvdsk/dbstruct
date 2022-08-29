use dbstruct::dbstruct;

struct CustomType {
    field: u8,
}

#[dbstruct(db=sled)]
struct Test {
    #[dbstruct(Default)]
    field: CustomType,
}

fn main() {}
