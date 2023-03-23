use super::{Bytes, FromBytes, ToBytes};
use crate::Result;

pub trait MemoryAdapter: Default {
    fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: FromBytes;

    fn put<V>(&mut self, key: &str, value: V) -> Option<Bytes>
    where
        V: ToBytes;

    fn contains_key(&self, key: &str) -> bool;
}
