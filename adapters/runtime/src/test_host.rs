use std::collections::HashMap;

use futures::lock::Mutex;
use lazy_static::lazy_static;

use crate::{HostAdapter, Result};

lazy_static! {
    #[derive(Clone, Default)]
    static ref HASHMAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());

}

#[derive(Clone, Default)]
pub struct HostTestAdapters;

impl HostTestAdapters {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let db = HASHMAP.lock().await;
        let value = db.get(key);
        Ok(value.cloned())
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut db = HASHMAP.lock().await;
        db.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn fetch(&mut self, url: &str) -> Result<String> {
        Ok(reqwest::get(url).await.unwrap().text().await?)
    }
}

pub struct RuntimeTestAdapter;

#[async_trait::async_trait]
impl HostAdapter for RuntimeTestAdapter {
    async fn db_get(key: &str) -> Result<Option<String>> {
        let host = HostTestAdapters::default();
        let result = host.get(key).await?;
        Ok(result)
    }

    async fn db_set(key: &str, value: &str) -> Result<()> {
        let host = HostTestAdapters::default();
        host.set(key, value).await?;
        Ok(())
    }

    async fn http_fetch(url: &str) -> Result<String> {
        let mut host = HostTestAdapters::default();
        let result = host.fetch(url).await?;
        Ok(result)
    }
}