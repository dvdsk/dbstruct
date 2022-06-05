#![allow(dead_code)]

mod dbstruct {
    use std::marker::PhantomData;

    pub struct Vec<T> {
        phantom: PhantomData<T>,
    }

    impl<T> Vec<T> {
        pub fn new() -> Self {
            Self {
                phantom: PhantomData,
            }
        }
    }

    pub struct Bool;
}

struct Song;
struct Preferences;

// macro input:
struct MacroInput {
    // lets say the actual name is Db
    queue: Vec<Song>,
    playing: bool,
    preferences: Preferences,
}

// start macro output
mod dbstruct_derived_for_db {
    use super::*;

    pub mod wrap {
        pub struct Preferences;

        impl Preferences {
            pub fn get(&self) -> super::super::Preferences {
                super::Preferences
            }
        }
    }

    pub(super) struct Db {
        // name of org struc
        pub queue: dbstruct::Vec<Song>,
        pub playing: dbstruct::Bool,
        pub preferences: wrap::Preferences,
    }

    impl Db {
        pub fn new() -> Db {
            Self {
                queue: dbstruct::Vec::new(),
                playing: dbstruct::Bool,
                preferences: wrap::Preferences,
            }
        }
    }
}

use dbstruct_derived_for_db::Db;
// end macro output

pub fn main() {
    let mut db = Db::new();

    db.preferences.get();
    println!("Hello, world!");
}
