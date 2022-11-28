/// A communication layer between Actix and the runtime
use actix::prelude::*;
use seda_runtime::{HostAdapter, RuntimeError};

use crate::host::{DatabaseGet, DatabaseSet, Host, HttpFetch};

pub struct RuntimeAdapter;

#[async_trait::async_trait]
impl HostAdapter for RuntimeAdapter {
    async fn db_get(key: &str) -> Result<Option<String>, RuntimeError> {
        let host_actor = Host::from_registry();

        let result = host_actor
            .send(DatabaseGet { key: key.to_string() })
            .await
            .unwrap()
            .unwrap();

        Ok(result)
    }

    async fn db_set(key: &str, value: &str) -> Result<(), RuntimeError> {
        let host_actor = Host::from_registry();

        host_actor
            .send(DatabaseSet {
                key:   key.to_string(),
                value: value.to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        Ok(())
    }

    async fn http_fetch(url: &str) -> Result<String, RuntimeError> {
        let host_actor = Host::from_registry();

        let result = host_actor.send(HttpFetch { url: url.to_string() }).await.unwrap();

        Ok(result)
    }
}
