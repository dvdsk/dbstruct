#[dbstruct::dbstruct(db=trait)]
pub struct Test {
    the_awnser: Option<u8>,
    the_question: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ds = sled::Config::default()
        .temporary(true)
        .open()?
        .open_tree("MacroInput")?;
    let db = Test::new(ds).unwrap();
    db.the_awnser().set(&42).unwrap();
    db.the_question()
        .set(&"The Ultimate Question".to_owned())
        .unwrap();

    Ok(())
}
