use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::ExtendError;
use crate::{DataStore, TryExtend};

use super::Map;

/// Inserts all new key-values from the iterator and replaces values with
/// existing keys with new values returned from the iterator.
impl<Key, Value, DS> TryExtend<(Key, Value)> for Map<'_, Key, Value, DS>
where
    DS: DataStore,
    Key: Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
{
    type Error = crate::Error<DS::Error>;

    fn try_extend<I>(
        &mut self,
        iter: I,
    ) -> Result<(), ExtendError<I::Item, I::IntoIter, Self::Error>>
    where
        I: IntoIterator<Item = (Key, Value)>,
    {
        let mut iter = iter.into_iter();
        loop {
            let Some((key, value)) = iter.next() else {
                return Ok(());
            };

            if let Err(error) = self.insert(&key, &value) {
                return Err(ExtendError {
                    unadded: (key, value),
                    iter,
                    error,
                });
            }
        }
    }
}


