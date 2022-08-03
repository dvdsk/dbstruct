use dbstruct::dbstruct;

#[dbstruct]
struct Test {
    #[dbstruct(NotAnnotation)]
    field: u8,
}

fn main() {}
