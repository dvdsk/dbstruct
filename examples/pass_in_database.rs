#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    the_awnser: Option<u8>,
    the_question: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = sled::Config::default().temporary(true).open()?;
    {
        let db = Test::open_db(db.clone())?;
        db.the_awnser().set(Some(&42))?;
        db.the_question().set(Some("The Ultimate Question"))?;
    }

    let tree = db.open_tree("MacroInput")?;
    {
        let db = Test::open_tree(tree)?;
        db.the_awnser().set(Some(&42))?;
        db.the_question().set(Some("The Ultimate Question"))?;
    }

    Ok(())
}
