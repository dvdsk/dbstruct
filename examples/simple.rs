use serde::{Serialize, Deserialize};

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
    // we dont need HashMap to be in scope as dbstruct 
    // will not use that type
    computers: HashMap<String, Computer>,
}

fn main() {
    let dir = tempdir::TempDir::new("dbstruct_examples").unwrap();
    let path = dir.path().join("advanced");
    let db = Test::new(path).unwrap();

    // we can store simple fields
    assert_eq!(None, db.the_awnser().get().unwrap());
    db.the_awnser().set(&42).unwrap();
    assert_eq!(Some(42u8), db.the_awnser().get().unwrap());

    // the vector is still empty
    assert_eq!(None, db.questions().pop().unwrap());

    // we push some elements
    db.questions().push("What is Life".to_owned()).unwrap();
    db.questions().push("What is the Universe".to_owned()).unwrap();
    db.questions().push("What is Everything".to_owned()).unwrap();

    // check the second element
    db.questions().pop().unwrap();
    let second = db.questions().pop().unwrap(); // we ignore the last element
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
        awnser: None, // unknown .... (go read: `The Restaurant at the End of the Universe`)
    };

    // Using a String as key is a little unergonomic as the insert function wants
    // values by reference. We need &String instead of &str.
    db.computers().insert(&"Deep Thought".to_owned(), &deep_thought).unwrap();
    db.computers().insert(&"Earth".to_owned(), &a_planet).unwrap();

    let earth = db.computers().get(&"Earth".to_owned()).unwrap();
    assert_eq!(earth.unwrap().secret, true);
}
