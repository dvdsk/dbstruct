use serde::{de::DeserializeOwned, Serialize};

use crate::traits::BytesStore;

pub struct Sled {
    tree: sled::Tree,
}

impl BytesStore for Sled {
    type Error = sled::Error;
    type Bytes = sled::IVec;

    fn get(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        self.tree.get(key)
    }

    fn remove(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        self.tree.remove(key)
    }

    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        self.tree.insert(key, val)
    }

    // fn atomic_update<BArg, BRet>(
    //     &self,
    //     key: &[u8],
    //     op: impl FnMut(Option<BArg>) -> BRet,
    // ) -> Result<(), Self::Error>
    // where
    //     BArg: AsRef<[u8]>,
    //     BRet: AsRef<[u8]> {

    //     self.tree.fetch_and_update(key, op).map(|_| ())
    // }

    fn conditional_update(
        &self,
        key: &[u8],
        new: &[u8],
        expected: &[u8],
    ) -> Result<(), Self::Error> {
        todo!()
    }

}
