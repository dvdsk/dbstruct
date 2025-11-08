#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    the_awnser: Option<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Test::open_path("trivial_example")?;
    if let Err(e) = db.the_awnser().set(&42) {
        match e {
            dbstruct::Error::Database(sled::Error::Io(e)) => eprintln!("io error in sled: {e}"),
            dbstruct::Error::Database(other_sled_issue) => {
                eprintln!("non io error in sled: {other_sled_issue}")
            }
            dbstruct::Error::SerializingKey(e) | dbstruct::Error::SerializingValue(e) => {
                eprintln!("serialization issue: {e}")
            }
            dbstruct::Error::DeSerializingKey(e) | dbstruct::Error::DeSerializingVal(e) => {
                eprintln!("serialization issue: {e}")
            }
        }
    }

    Ok(())
}
