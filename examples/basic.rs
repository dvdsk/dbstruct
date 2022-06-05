use std::collections::HashMap;

use dbstruct::dbstruct;
use serde::{Deserialize, Serialize};

use self::some::lib::ExampleType;

mod some {
    use super::*;
    pub mod lib {
        use super::*;
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        pub struct ExampleType(pub u32);
    }
}

#[dbstruct]
struct State {
    // position: Option<ExampleType>,
    // #[dbstruct(default(r#"String::from("test")"#))] // YAY this is possible
    // #[dbstruct(no_idx)] // YAY this is possible
    // feed: String,

    // #[dbstruct] // YAY this is possible
    // numbers: Vec<u8>,
    mappy: HashMap<u8,u16>,
}

fn main() {
    let state = State::test().unwrap();

    // let position = Some(ExampleType(5));
    // state.set_position(&position).unwrap();
    // let feed = Some("Hello".to_owned());
    // state.set_feed(&feed).unwrap();

    // assert_eq!(position, state.position().unwrap());
    // assert_eq!(feed, state.feed().unwrap());
}
