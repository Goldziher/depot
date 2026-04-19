use async_trait::async_trait;
use bytes::Bytes;
use opendal::Operator;

use depot_core::error::{DepotError, Result};
use depot_core::ports::StoragePort;

/// Storage backend backed by OpenDAL.
pub struct OpenDalStorage {
    operator: Operator,
}

impl OpenDalStorage {
    pub fn new(operator: Operator) -> Self {
        Self { operator }
    }
}

#[async_trait]
impl StoragePort for OpenDalStorage {
    async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        match self.operator.read(key).await {
            Ok(data) => Ok(Some(data.to_bytes())),
            Err(e) if e.kind() == opendal::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DepotError::Storage(e.to_string())),
        }
    }

    async fn put(&self, key: &str, data: Bytes) -> Result<()> {
        self.operator
            .write(key, data)
            .await
            .map_err(|e| DepotError::Storage(e.to_string()))
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.operator
            .exists(key)
            .await
            .map_err(|e| DepotError::Storage(e.to_string()))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.operator
            .delete(key)
            .await
            .map_err(|e| DepotError::Storage(e.to_string()))
    }

    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        let entries = self
            .operator
            .list(prefix)
            .await
            .map_err(|e| DepotError::Storage(e.to_string()))?;

        Ok(entries.into_iter().map(|e| e.path().to_string()).collect())
    }
}
