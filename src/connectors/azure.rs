use anyhow::Result;
use async_trait::async_trait;
use azure_core::auth::TokenCredential;
use azure_identity::ClientSecretCredential;
use azure_security_keyvault::prelude::*;
use std::sync::Arc;

pub struct AzureConnector {
    credential: Arc<dyn TokenCredential>,
    vault_url: String,
}

impl AzureConnector {
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let client_id = config
            .connector_auth
            .azure_client_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AZURE_CLIENT_ID not configured"))?;

        let client_secret = config
            .connector_auth
            .azure_client_secret
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AZURE_CLIENT_SECRET not configured"))?;

        let tenant_id = config
            .connector_auth
            .azure_tenant_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("AZURE_TENANT_ID not configured"))?;

        let vault_name = std::env::var("AZURE_VAULT_NAME")
            .map_err(|_| anyhow::anyhow!("AZURE_VAULT_NAME environment variable not set"))?;

        let vault_url = format!("https://{}.vault.azure.net", vault_name);

        let http_client = azure_core::new_http_client();
        let authority_host = "https://login.microsoftonline.com";
        
        let credential: Arc<dyn TokenCredential> = Arc::new(
            ClientSecretCredential::new(
                http_client,
                authority_host.parse().unwrap(),
                tenant_id.clone(),
                client_id.clone(),
                client_secret.clone(),
            )
        );

        Ok(Self { credential, vault_url })
    }
}

#[async_trait]
impl crate::connectors::Connector for AzureConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        let client = SecretClient::new(&self.vault_url, self.credential.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create Azure Key Vault client: {}", e))?;

        client
            .set(name, value)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set secret in Azure Key Vault: {}", e))?;

        Ok(())
    }

    async fn get_secret(&self, name: &str) -> Result<String> {
        let client = SecretClient::new(&self.vault_url, self.credential.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create Azure Key Vault client: {}", e))?;

        let secret = client
            .get(name)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get secret from Azure Key Vault: {}", e))?;

        Ok(secret.value.to_string())
    }

    async fn trigger_refresh(&self, service: Option<&str>) -> Result<()> {
        if let Some(svc) = service {
            println!("ℹ️  Would trigger refresh for Azure service: {}", svc);
            println!("   (e.g., App Service restart, Container Apps revision)");
            println!("   Note: Automatic refresh not implemented - manually restart your service");
        }

        Ok(())
    }
}
