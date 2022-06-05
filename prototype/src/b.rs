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

    pub struct DefaultValue<T> {
        phantom: PhantomData<T>,
    }

    impl<T> DefaultValue<T> {
        pub fn new() -> Self {
            Self { 
                phantom: PhantomData,
            }
        }
    }

    pub struct DefaultTrait<T> {
        phantom: PhantomData<T>,
    }

    impl<T: Default> DefaultTrait<T> {
        pub fn new() -> Self {
            Self { 
                phantom: PhantomData,
            }
        }

        pub fn get(&self) -> T {
            T::default()
        }
    }

    pub struct Option<T> {
        phantom: PhantomData<T>,
    }

    impl<T> Option<T> {
        pub fn new() -> Self {
            Self {
                phantom: PhantomData
            }
        }
    }
}

struct Song;
#[derive(Default)]
struct Preferences;
struct Account;

// macro input:
struct MacroInput {
    // lets say the actual name is Db
    pub queue: Vec<Song>,
    //#[dbstruct(Default("true")]
    pub playing: bool,
    //#[dbstruct(Default)]
    pub preferences: Preferences,
    pub account: Option<Account>
}

// note only make fields pub if they are in the
// original struct
// start macro output
pub(super) struct Db {
    // name of org struc
    pub queue: dbstruct::Vec<Song>,
    pub playing: dbstruct::DefaultValue<bool>,
    /// this is documentation test, can be generated too
    pub preferences: dbstruct::DefaultTrait<Preferences>,
    pub account: dbstruct::Option<Account>,
}

impl Db {
    pub fn new() -> Db {
        Self {
            queue: dbstruct::Vec::new(),
            playing: dbstruct::DefaultValue::new(),
            preferences: dbstruct::DefaultTrait::new(),
            account: dbstruct::Option::new(),
        }
    }
}
// end macro output

pub fn main() {
    let mut db = Db::new();

    db.preferences.get();
    println!("Hello, world!");
}
