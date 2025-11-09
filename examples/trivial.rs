#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    #[dbstruct(Default = "Some(42)")]
    the_awnser: Option<u8>,
    the_question: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Test::open_path("trivial_example")?;
    db.the_awnser().set(&Some(42))?;
    db.the_question().set(Some("The Ultimate Question"))?;

    // dropping the db here simulates the program
    // stopping and restarting
    std::mem::drop(db);
    let db = Test::open_path("trivial_example")?;

    let the_awnser = db.the_awnser().get()?;
    assert_eq!(the_awnser, Some(42));

    let the_question = db.the_question().get()?;
    assert_eq!(the_question, Some("The Ultimate Question".to_owned()));

    Ok(())
}
