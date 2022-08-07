use crate::ByteStore;
use rocksdb::{ThreadMode, TransactionDB};

// default off, needs libclang-dev package on ubuntu

/* TODO: when GATS stabalize we can use rocksdb::get_pinned
and save an allocation. Alternatively we could implement
DataStore directly and use pinnend in that implementation
<07-08-22, dvdsk> */

impl<TH: ThreadMode> ByteStore for TransactionDB<TH> {
    type Error = rocksdb::Error;
    type Bytes = Vec<u8>;

    fn get(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        self.get(key)
    }

    fn remove(&self, key: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        use rocksdb::ErrorKind::*;

        let txn = self.transaction();
        let get_res = txn.get(key);
        let del_res = txn.delete(key);
        txn.commit()?;

        let val = match get_res {
            Ok(None) => return Ok(None),
            Err(e) if e.kind() == NotFound => return Ok(None),
            Err(e) => return Err(e),
            Ok(Some(val)) => val,
        };

        match del_res {
            Err(e) if e.kind() == NotFound => return Ok(None),
            Err(e) => return Err(e),
            Ok(()) => return Ok(Some(val)),
        }
    }

    fn insert(&self, key: &[u8], val: &[u8]) -> Result<Option<Self::Bytes>, Self::Error> {
        use rocksdb::ErrorKind::*;

        let txn = self.transaction();
        let get_res = txn.get(key);
        let del_res = txn.put(key, val);
        txn.commit()?;

        del_res?;
        match get_res {
            Ok(None) => Ok(None),
            Err(e) if e.kind() == NotFound => Ok(None),
            Err(e) => Err(e),
            Ok(Some(val)) => Ok(Some(val)),
        }
    }
}
