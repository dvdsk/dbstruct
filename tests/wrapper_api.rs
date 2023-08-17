use std::error::Error;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use dbstruct::traits::data_store;
use dbstruct::traits::DataStore;
use dbstruct::wrapper;
use serde::{Deserialize, Serialize};
use tracing::instrument;

mod setup_tracing;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Song;
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct Preferences;
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Account;

const PLAYING_DEFAULT: bool = true;

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
pub struct MacroOutput<DS: DataStore> {
    ds: DS,
    queue_len: Arc<AtomicUsize>,
}

impl<DS> MacroOutput<DS>
where
    DS: DataStore + data_store::Ordered + Clone,
{
    pub fn new(ds: DS) -> Result<Self, dbstruct::Error<<DS as DataStore>::Error>> {
        let queue_len = data_store::Ordered::get_lt(&ds, &(1+1))?
            .map(|(len, _): (usize, Song)| len)
            .unwrap_or(0);
        tracing::debug!("opening vector queue with len: {queue_len}");
        Ok(Self {
            ds,
            queue_len: Arc::new(AtomicUsize::new(queue_len)),
        })
    }

    #[instrument(skip_all)]
    pub fn queue(&self) -> wrapper::Vec<Song, DS> {
        wrapper::Vec::new(self.ds.clone(), 1, self.queue_len.clone())
    }
    #[instrument(skip_all)]
    pub fn playing(&self) -> wrapper::DefaultValue<bool, DS> {
        wrapper::DefaultValue::new(self.ds.clone(), 2, PLAYING_DEFAULT)
    }
    #[instrument(skip_all)]
    pub fn preferences(&self) -> wrapper::DefaultTrait<Preferences, DS> {
        wrapper::DefaultTrait::new(self.ds.clone(), 3)
    }
    #[instrument(skip_all)]
    pub fn account(&self) -> wrapper::OptionValue<Account, DS> {
        wrapper::OptionValue::new(self.ds.clone(), 4)
    }
}
// end macro output

pub fn main() -> Result<(), Box<dyn Error>> {
    setup_tracing::setup("");
    let ds = sled::Config::default()
        .temporary(true)
        .open()?
        .open_tree("MacroInput")?;
    let db = MacroOutput::new(ds)?;

    let last = db.queue().pop()?;
    assert_eq!(last, None);
    db.queue().push(&Song {})?;
    let last = db.queue().pop()?;
    assert_eq!(last, Some(Song {}));

    let playing = db.playing().get()?;
    assert_eq!(playing, PLAYING_DEFAULT);

    let preferences = db.preferences().get()?;
    assert_eq!(preferences, Default::default());

    db.account().set(&Account {})?;
    let account = db.account().get()?;
    assert_eq!(account, Some(Account {}));

    db.account().conditional_update(Account {}, Account {})?;

    Ok(())
}
