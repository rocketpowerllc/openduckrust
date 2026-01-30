//! Trait-based dependency injection for AWS Services and AI Providers.

use async_trait::async_trait;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn get_item(&self, table: &str, key: &str, tenant_id: &str) -> anyhow::Result<Option<serde_json::Value>>;
    async fn put_item(&self, table: &str, item: serde_json::Value) -> anyhow::Result<()>;
    async fn query_by_tenant(&self, table: &str, tenant_id: &str) -> anyhow::Result<Vec<serde_json::Value>>;
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn invoke(&self, prompt: &str) -> anyhow::Result<String>;
}

// TODO: Implement DynamoDbProvider, BedrockProvider
