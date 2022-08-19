use dbstruct::dbstruct;

struct CustomType {
    field: u8,
}

#[dbstruct]
struct Test {
    #[dbstruct(Default)]
    field: CustomType,
}

fn main() {}
