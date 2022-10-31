use std::collections::HashMap;

use seda_runtime_macros::Adapter;

use crate::{DatabaseAdapter, HostAdapterTypes, HttpAdapter};

#[derive(Clone, Default, Adapter)]
#[adapter(
	database = DatabaseTestAdapter,
	http = HttpTestAdapter,
)]
pub struct TestAdapters;

#[derive(Default)]
pub struct DatabaseTestAdapter {
    data: HashMap<String, String>,
}

impl DatabaseAdapter for DatabaseTestAdapter {
    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    fn get_all(&self) -> HashMap<String, String> {
        self.data.clone()
    }
}

#[derive(Default)]
pub struct HttpTestAdapter;

#[async_trait::async_trait]
impl HttpAdapter for HttpTestAdapter {
    async fn fetch(&mut self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        reqwest::get(url).await
    }
}
