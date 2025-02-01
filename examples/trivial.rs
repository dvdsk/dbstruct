#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    the_awnser: Option<u8>,
    the_question: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Test::new("trivial_example")?;
    db.the_awnser().set(&42)?;
    db.the_question().set("The Ultimate Question")?;

    // dropping the db here simulates the program
    // stopping and restarting
    std::mem::drop(db);
    let db = Test::new("trivial_example")?;

    let the_awnser = db.the_awnser().get()?;
    assert_eq!(the_awnser, Some(42));

    let the_question = db.the_question().get()?;
    assert_eq!(the_question, Some("The Ultimate Question".to_owned()));

    Ok(())
}
