use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use dbstruct::wrappers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Song;
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
    queue_len: Arc<AtomicUsize>,
}

impl MacroOutput {
    pub fn new() -> Result<Self, dbstruct::Error<sled::Error>> {
        let queue_len = 0; // TODO: decide where to store 
                           // this in DB and how to load it
        Ok(Self {
            ds: sled::Config::default()
                .temporary(true)
                .open()?
                .open_tree("MacroInput")?,
            queue_len: Arc::new(AtomicUsize::new(queue_len)),
        })
    }

    pub fn queue(&self) -> wrappers::Vec<Song, sled::Tree> {
        wrappers::Vec::new(self.ds.clone(), 1, self.queue_len.clone())
    }
    pub fn playing(&self) -> wrappers::DefaultValue<bool, sled::Tree> {
        wrappers::DefaultValue::new(self.ds.clone(), 2, false)
    }
    pub fn preferences(&self) -> wrappers::DefaultTrait<Preferences, sled::Tree> {
        wrappers::DefaultTrait::new(self.ds.clone(), 3)
    }
    pub fn account(&self) -> wrappers::OptionValue<Account, sled::Tree> {
        wrappers::OptionValue::new(self.ds.clone(), 4)
    }
}
// end macro output

pub fn main() {
    let db = MacroOutput::new().unwrap();

    let _preferences = db.preferences().get().unwrap();
    println!("Hello, world!");
}
