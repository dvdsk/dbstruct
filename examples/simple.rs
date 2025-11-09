use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Computer {
    secret: bool,
    question: String,
    awnser: Option<usize>,
}

#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    the_awnser: Option<u8>,
    questions: Vec<String>,
    // we do not need HashMap to be in scope as dbstruct
    // will not use that type
    computers: HashMap<String, Computer>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir::TempDir::new("dbstruct_examples")?;
    let path = dir.path().join("advanced");
    let db = Test::open_path(path)?;

    // we can store simple fields
    assert_eq!(None, db.the_awnser().get()?);
    db.the_awnser().set(Some(&42))?;
    assert_eq!(Some(42u8), db.the_awnser().get()?);

    // the vector is still empty
    assert_eq!(None, db.questions().pop()?);

    // we push some elements
    db.questions().push("What is Life")?;
    db.questions().push("What is the Universe")?;
    db.questions().push("What is Everything")?;

    // check the second element
    db.questions().pop()?;
    let second = db.questions().pop()?; // we ignore the last element
    assert_eq!(Some("What is the Universe".to_owned()), second);

    // we can also use custom types as long as they implement
    // serde Serialize and Deserialize.
    let deep_thought = Computer {
        secret: false,
        question: "The Ultimate Question of Life, the Universe, and Everything".to_owned(),
        awnser: Some(42),
    };

    let a_planet = Computer {
        secret: true,
        question: "What is The Ultimate Question".to_owned(),
        awnser: None, // unknown ... (go read: `The Restaurant at the End of the Universe`)
    };

    db.computers().insert("Deep Thought", &deep_thought)?;
    db.computers().insert("Earth", &a_planet)?;

    let earth = db.computers().get("Earth")?;
    assert!(earth.unwrap().secret);

    Ok(())
}
