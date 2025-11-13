use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use uuid::Uuid;

pub struct CredentialCache {
    manager: ConnectionManager,
    ttl_seconds: usize,
}

impl CredentialCache {
    pub async fn new(redis_url: &str, ttl_seconds: usize) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self {
            manager,
            ttl_seconds,
        })
    }

    fn cache_key(workspace_id: &Uuid, provider: &str, secret_name: &str) -> String {
        format!("cred:{}:{}:{}", workspace_id, provider, secret_name)
    }

    pub async fn get(
        &mut self,
        workspace_id: &Uuid,
        provider: &str,
        secret_name: &str,
    ) -> Result<Option<String>> {
        let key = Self::cache_key(workspace_id, provider, secret_name);
        let value: Option<String> = self.manager.get(&key).await?;
        Ok(value)
    }

    pub async fn set(
        &mut self,
        workspace_id: &Uuid,
        provider: &str,
        secret_name: &str,
        value: &str,
    ) -> Result<()> {
        let key = Self::cache_key(workspace_id, provider, secret_name);
        self.manager
            .set_ex(&key, value, self.ttl_seconds as u64)
            .await?;
        Ok(())
    }

    pub async fn invalidate(
        &mut self,
        workspace_id: &Uuid,
        provider: &str,
        secret_name: &str,
    ) -> Result<()> {
        let key = Self::cache_key(workspace_id, provider, secret_name);
        let _: () = self.manager.del(&key).await?;
        Ok(())
    }
}
