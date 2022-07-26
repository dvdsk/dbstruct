use crate::traits::{byte_store, ByteStore};

pub struct Sled {
    tree: sled::Tree,
}

impl ByteStore for Sled {
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
}

impl byte_store::Atomic for Sled {
    fn atomic_update(
        &self,
        key: &[u8],
        op: impl FnMut(Option<&[u8]>) -> Option<Vec<u8>>,
    ) -> Result<(), Self::Error> {
        self.tree.fetch_and_update(key, op).map(|_| ())
    }

    fn conditional_update(
        &self,
        key: &[u8],
        new: &[u8],
        expected: &[u8],
    ) -> Result<(), Self::Error> {
        let _ignore_compare_and_swap_res =
            self.tree.compare_and_swap(key, Some(expected), Some(new))?;
        Ok(())
    }
}
