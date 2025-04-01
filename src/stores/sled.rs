use crate::traits::{byte_store, ByteStore};

impl ByteStore for sled::Tree {
    type DbError = sled::Error;
    type Bytes = sled::IVec;

    fn get(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError> {
        self.get(key)
    }

    fn remove(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError> {
        self.remove(key)
    }

    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Self::Bytes>, Self::DbError> {
        self.insert(key, val)
    }
}

impl byte_store::Atomic for sled::Tree {
    fn atomic_update(
        &self,
        key: &[u8],
        op: impl FnMut(Option<&[u8]>) -> Option<Vec<u8>>,
    ) -> Result<(), Self::DbError> {
        self.fetch_and_update(key, op).map(|_| ())
    }

    fn conditional_update(
        &self,
        key: &[u8],
        new: &[u8],
        expected: &[u8],
    ) -> Result<(), Self::DbError> {
        let _ignore_compare_and_swap_res = self.compare_and_swap(key, Some(expected), Some(new))?;
        Ok(())
    }
}

impl byte_store::Ordered for sled::Tree {
    fn get_lt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::DbError> {
        self.get_lt(key)
    }
    fn get_gt(&self, key: &[u8]) -> Result<Option<(Self::Bytes, Self::Bytes)>, Self::DbError> {
        self.get_gt(key)
    }
}

impl byte_store::Ranged for sled::Tree {
    type Key = Vec<u8>;
    type Iter = sled::Iter;

    fn range<'a>(&self, range: impl std::ops::RangeBounds<Self::Key>) -> Self::Iter {
        self.range(range)
    }
}
