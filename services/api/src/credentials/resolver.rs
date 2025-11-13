use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::credentials::cache::CredentialCache;
use crate::credentials::modes::CredentialMode;
use crate::supabase::SupabaseClient;
use crate::vault::storage::VaultStorage;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;

pub struct CredentialResolver {
    client: SupabaseClient,
    vault: VaultStorage,
    cache: CredentialCache,
}

impl CredentialResolver {
    pub fn new(client: SupabaseClient, vault: VaultStorage, cache: CredentialCache) -> Self {
        Self {
            client,
            vault,
            cache,
        }
    }

    async fn get_provider_mode(
        &self,
        workspace_id: &Uuid,
        provider: &str,
    ) -> Result<CredentialMode> {
        let db_client = self.client.get_client().await?;

        let stmt = db_client
            .prepare(
                "SELECT mode FROM provider_configs
                 WHERE workspace_id = $1 AND provider = $2",
            )
            .await?;

        let rows = db_client.query(&stmt, &[workspace_id, &provider]).await?;

        if rows.is_empty() {
            return Ok(CredentialMode::Hosted);
        }

        let mode_str: String = rows[0].get(0);
        mode_str.parse()
    }

    pub async fn resolve(
        &mut self,
        workspace_id: &Uuid,
        provider: &str,
        secret_name: &str,
    ) -> Result<String> {
        if let Some(cached) = self.cache.get(workspace_id, provider, secret_name).await? {
            tracing::debug!("Cache hit for credential");
            return Ok(cached);
        }

        let mode = self.get_provider_mode(workspace_id, provider).await?;

        let credential = match mode {
            CredentialMode::Hosted => {
                self.resolve_hosted(workspace_id, provider, secret_name)
                    .await?
            }
            CredentialMode::OAuth => {
                tracing::warn!("OAuth mode not yet implemented");
                anyhow::bail!("OAuth mode not yet implemented")
            }
            CredentialMode::Kms => {
                tracing::warn!("KMS mode not yet implemented");
                anyhow::bail!("KMS mode not yet implemented")
            }
            CredentialMode::ApiKey => {
                tracing::warn!("API key mode not yet implemented");
                anyhow::bail!("API key mode not yet implemented")
            }
        };

        self.cache
            .set(workspace_id, provider, secret_name, &credential)
            .await?;

        Ok(credential)
    }

    async fn resolve_hosted(
        &self,
        workspace_id: &Uuid,
        provider: &str,
        secret_name: &str,
    ) -> Result<String> {
        let mut attempt = 0;

        loop {
            match self
                .vault
                .get_credential(*workspace_id, provider, secret_name)
                .await
            {
                Ok(Some(credential)) => return Ok(credential),
                Ok(None) => anyhow::bail!("Credential not found"),
                Err(e) => {
                    attempt += 1;
                    if attempt >= MAX_RETRIES {
                        return Err(e);
                    }

                    let backoff = INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1);
                    tracing::warn!(
                        "Credential resolution failed (attempt {}), retrying in {}ms",
                        attempt,
                        backoff
                    );
                    sleep(Duration::from_millis(backoff)).await;
                }
            }
        }
    }
}
