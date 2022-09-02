use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Computer {
    secret: bool,
    question: String,
    awnser: Option<usize>,
}

#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    #[dbstruct(Default)]
    the_awnser: u8,
    #[dbstruct(Default = "vec![\"What is Life\".to_owned()]")]
    questions: Vec<String>,
    // now we need HashMap in scope as computers is now
    // a single value of type `HashMap` we set and get
    #[dbstruct(Default)]
    computers: HashMap<String, Computer>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir::TempDir::new("dbstruct_examples").unwrap();
    let path = dir.path().join("advanced");

    let db = Test::new(&path).unwrap();

    // we get the `Default` value for u8
    assert_eq!(0, db.the_awnser().get()?);
    db.the_awnser().set(&42)?;

    // the vectors now is a single value
    let mut questions = db.questions().get()?;
    // it is initialized using the expression we set
    assert_eq!(vec!["What is Life".to_owned()], questions);

    questions.push("What is the Universe".to_owned());
    questions.push("What is Everything".to_owned());
    db.questions().set(&questions)?;

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
        awnser: None, // unknown .... (go read: `The Restaurant at the End of the Universe`)
    };
    let mut computers = HashMap::new();
    computers.insert("Deep Thought".to_owned(), deep_thought);
    computers.insert("Earth".to_owned(), a_planet);
    db.computers().set(&computers)?;

    // dropping the db here simulates the program 
    // stopping and restarting
    std::mem::drop(db); // this closes the db
    let db = Test::new(&path).unwrap();

    assert_eq!(42u8, db.the_awnser().get()?);

    let second = &db.questions().get()?[1];
    assert_eq!(&"What is the Universe".to_owned(), second);

    let computers = db.computers().get().unwrap();
    let earth = computers.get(&"Earth".to_owned());
    assert_eq!(earth.unwrap().secret, true);

    Ok(())
}
