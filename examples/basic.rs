use dbstruct::wrappers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Song;
#[derive(Serialize, Deserialize, Default)]
pub struct Preferences;
#[derive(Serialize, Deserialize)]
pub struct Account;

// macro input:
#[allow(dead_code)]
struct MacroInput {
    // lets say the actual name is Db
    pub queue: Vec<Song>,
    //#[dbstruct(Default("true")]
    pub playing: bool,
    //#[dbstruct(Default)]
    pub preferences: Preferences,
    pub account: Option<Account>,
}

// note only make fields pub if they are in the
// original struct
// start macro output
// note the macro would use absolute paths for everything
pub struct MacroOutput {
    ds: sled::Tree,
}

impl MacroOutput {
    pub fn new() -> Result<Self, dbstruct::Error<sled::Error>> {
        Ok(Self {
            ds: sled::Config::default()
                .temporary(true)
                .open()?
                .open_tree("MacroInput")?,
        })
    }

    // pub fn queue(&self) -> wrappers::Vec<Song, sled::Tree> {
    //     wrappers::Vec::new(self.ds, 0)
    // }
    pub fn playing(&self) -> wrappers::DefaultValue<bool, sled::Tree> {
        wrappers::DefaultValue::new(self.ds.clone(), 1, false)
    }
    pub fn preferences(&self) -> wrappers::DefaultTrait<Preferences, sled::Tree> {
        wrappers::DefaultTrait::new(self.ds.clone(), 2)
    }
    pub fn account(&self) -> wrappers::OptionValue<Account, sled::Tree> {
        wrappers::OptionValue::new(self.ds.clone(), 3)
    }
}
// end macro output

pub fn main() {
    let db = MacroOutput::new().unwrap();

    let _preferences = db.preferences().get().unwrap();
    println!("Hello, world!");
}
