use std::{collections::HashMap, ops::Deref};

use super::{Bytes, FromBytes, MemoryAdapter, ToBytes};
use crate::Result;

#[derive(Debug, Default)]
pub struct InMemory {
    memory: HashMap<String, Bytes>,
}

impl MemoryAdapter for InMemory {
    fn get<O>(&self, key: &str) -> Result<Option<O>>
    where
        O: FromBytes,
    {
        Ok(self.memory.get(key).map(|b| O::from_bytes(b.deref())).transpose()?)
    }

    fn put<V>(&mut self, key: &str, value: V) -> Option<Bytes>
    where
        V: ToBytes,
    {
        self.memory.insert(key.into(), value.to_bytes())
    }

    fn contains_key(&self, key: &str) -> bool {
        self.memory.contains_key(key)
    }
}
