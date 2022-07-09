use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{CompareAndSwapError, Error};

pub struct DefaultValue<T> {
    default_value: T,
    tree: sled::Tree,
    key: u8,
}

impl<T: Serialize + DeserializeOwned + Clone> DefaultValue<T> {
    pub fn new(tree: sled::Tree, key: u8, default_value: T) -> Self {
        Self {
            default_value,
            tree,
            key,
        }
    }

    pub fn set(&mut self, value: &T) -> Result<(), Error> {
        let bytes = bincode::serialize(value).map_err(Error::Serializing)?;
        self.tree.insert([self.key], bytes)?;
        Ok(())
    }

    pub fn get(&self) -> Result<T, Error> {
        match self.tree.get([self.key])? {
            Some(bytes) => Ok(bincode::deserialize(&bytes).map_err(Error::DeSerializing)?),
            None => Ok(self.default_value.clone()),
        }
    }

    pub fn update(&self, op: impl FnMut(T) -> T + Clone) -> Result<(), Error> {
        let mut res = Ok(());
        let update = |old: Option<&[u8]>| match old {
            None => {
                let default_val = self.default_value.clone();
                let new = op.clone()(default_val);
                match bincode::serialize(&new) {
                    Ok(new_bytes) => Some(new_bytes),
                    Err(e) => {
                        res = Err(Error::Serializing(e));
                        None
                    }
                }
            }
            Some(old) => match bincode::deserialize(old) {
                Err(e) => {
                    res = Err(Error::DeSerializing(e));
                    Some(old.to_vec())
                }
                Ok(v) => {
                    let new = op.clone()(v);
                    match bincode::serialize(&new) {
                        Ok(new_bytes) => Some(new_bytes),
                        Err(e) => {
                            res = Err(Error::Serializing(e));
                            Some(old.to_vec())
                        }
                    }
                }
            },
        };

        self.tree.update_and_fetch([self.key], update)?;
        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned + PartialEq> DefaultValue<T> {
    pub fn compare_and_swap(
        &self,
        old: T,
        new: T,
    ) -> Result<Result<(), CompareAndSwapError<T>>, Error> {
        // The default value is encoded as no value in the db. If the user is
        // comparing agains the old vale change the call in the array
        let default_val = &self.default_value;
        let old = if &old == default_val {
            None
        } else {
            let bytes = bincode::serialize(&old).map_err(Error::Serializing)?;
            Some(bytes)
        };

        // I save the default as None not to save space but keep initialization
        // fast, otherwise the default value would need to be written for each
        // dbstruct member. Therefore we do not take the time to encode the new
        // as None even if new is the default value
        let new = bincode::serialize(&new).map_err(Error::Serializing)?;
        let res = self.tree.compare_and_swap([self.key], old, Some(new))?;
        Ok(match res {
            Ok(()) => Ok(()),
            Err(e) => Err(e.try_into()?),
        })
    }
}
