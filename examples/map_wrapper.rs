#[dbstruct::dbstruct(db=sled)]
#[derive(Clone)]
pub struct Test {
    computers: HashMap<String, usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir::TempDir::new("dbstruct_examples")?;
    let path = dir.path().join("map_wrapper");

    let db = Test::new(&path)?;
    db.computers().insert("Deep Thought", &42)?;
    db.computers().insert("Colossus", &1944)?;
    db.computers().insert("ENIAC", &1946)?;
    db.computers().insert("System/360", &1953)?;
    db.computers().insert("DEC PDP-8", &1970)?;
    db.computers().insert("IBM PC", &1981)?;

    db.computers().remove("Deep Thought")?;
    let real_computers = db.computers().keys().collect::<Result<Vec<_>, _>>()?;
    assert!(!real_computers.contains(&"Deep Thought".to_owned()));

    Ok(())
}
