use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::supabase::SupabaseClient;
use crate::vault::encryption::VaultEncryption;

pub struct VaultStorage {
    client: SupabaseClient,
    encryption: VaultEncryption,
}

impl VaultStorage {
    pub fn new(client: SupabaseClient, encryption: VaultEncryption) -> Self {
        Self { client, encryption }
    }

    pub async fn store_credential(
        &self,
        workspace_id: Uuid,
        provider: &str,
        secret_name: &str,
        value: &str,
    ) -> Result<()> {
        let encrypted_value = self.encryption.encrypt(&workspace_id, value)?;

        let db_client = self.client.get_client().await?;

        let stmt = db_client
            .prepare(
                "INSERT INTO credentials (workspace_id, provider, secret_name, encrypted_value)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (workspace_id, provider, secret_name)
                 DO UPDATE SET encrypted_value = $4, updated_at = NOW()",
            )
            .await?;

        db_client
            .execute(
                &stmt,
                &[&workspace_id, &provider, &secret_name, &encrypted_value],
            )
            .await?;

        Ok(())
    }

    pub async fn get_credential(
        &self,
        workspace_id: Uuid,
        provider: &str,
        secret_name: &str,
    ) -> Result<Option<String>> {
        let db_client = self.client.get_client().await?;

        let stmt = db_client
            .prepare(
                "SELECT encrypted_value FROM credentials
                 WHERE workspace_id = $1 AND provider = $2 AND secret_name = $3 AND deleted_at IS NULL",
            )
            .await?;

        let rows = db_client
            .query(&stmt, &[&workspace_id, &provider, &secret_name])
            .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let encrypted_value: Vec<u8> = rows[0].get(0);
        let decrypted = self.encryption.decrypt(&workspace_id, &encrypted_value)?;

        Ok(Some(decrypted))
    }

    pub async fn update_credential(
        &self,
        workspace_id: Uuid,
        provider: &str,
        secret_name: &str,
        value: &str,
    ) -> Result<bool> {
        let encrypted_value = self.encryption.encrypt(&workspace_id, value)?;

        let db_client = self.client.get_client().await?;

        let stmt = db_client
            .prepare(
                "UPDATE credentials
                 SET encrypted_value = $4, updated_at = NOW()
                 WHERE workspace_id = $1 AND provider = $2 AND secret_name = $3 AND deleted_at IS NULL",
            )
            .await?;

        let rows_affected = db_client
            .execute(
                &stmt,
                &[&workspace_id, &provider, &secret_name, &encrypted_value],
            )
            .await?;

        Ok(rows_affected > 0)
    }

    pub async fn delete_credential(
        &self,
        workspace_id: Uuid,
        provider: &str,
        secret_name: &str,
    ) -> Result<bool> {
        let db_client = self.client.get_client().await?;

        let stmt = db_client
            .prepare(
                "UPDATE credentials
                 SET deleted_at = NOW()
                 WHERE workspace_id = $1 AND provider = $2 AND secret_name = $3 AND deleted_at IS NULL",
            )
            .await?;

        let rows_affected = db_client
            .execute(&stmt, &[&workspace_id, &provider, &secret_name])
            .await?;

        Ok(rows_affected > 0)
    }
}
