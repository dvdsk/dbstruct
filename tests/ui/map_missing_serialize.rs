
enum CustomKeyType {}
struct CustomValType;

fn main() {
    // ONCE the output changes and the span starts highlighting the 
    // key and value type in the struct we can close: 
    // https://github.com/dvdsk/dbstruct/issues/13

    #[dbstruct::dbstruct(db=sled)]
    struct PersistentData {
        the_map: HashMap<CustomKeyType, CustomValType>,
    }
}
