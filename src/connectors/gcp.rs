use anyhow::{Context, Result};
use async_trait::async_trait;
use google_secretmanager1::{
    api::{AddSecretVersionRequest, Secret},
    hyper, hyper_rustls, oauth2, SecretManager,
};

pub struct GcpConnector {
    hub: SecretManager<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    project_id: String,
}

impl GcpConnector {
    pub async fn new_async(config: &crate::config::Config) -> Result<Self> {
        let credentials_path = config
            .connector_auth
            .gcp_credentials_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("GOOGLE_APPLICATION_CREDENTIALS not configured"))?;

        let project_id = std::env::var("GCP_PROJECT_ID")
            .map_err(|_| anyhow::anyhow!("GCP_PROJECT_ID environment variable not set"))?;

        let service_account_key = oauth2::read_service_account_key(credentials_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read GCP credentials: {}", e))?;

        let auth = oauth2::ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to authenticate with GCP: {}", e))?;

        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| anyhow::anyhow!("Failed to configure TLS: {}", e))?
            .https_or_http()
            .enable_http1()
            .build();

        let hub = SecretManager::new(hyper::Client::builder().build(connector), auth);

        Ok(Self { hub, project_id })
    }

    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;
        
        rt.block_on(Self::new_async(config))
    }
}

#[async_trait]
impl crate::connectors::Connector for GcpConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        let parent = format!("projects/{}", self.project_id);
        let secret_path = format!("{}/secrets/{}", parent, name);

        let result = self.hub.projects().secrets_get(&secret_path).doit().await;

        match result {
            Ok(_) => {
                let payload = AddSecretVersionRequest {
                    payload: Some(google_secretmanager1::api::SecretPayload {
                        data: Some(value.as_bytes().to_vec()),
                        ..Default::default()
                    }),
                };

                self.hub
                    .projects()
                    .secrets_add_version(payload, &secret_path)
                    .doit()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to add secret version in GCP: {}", e))?;
            }
            Err(_) => {
                let secret = Secret {
                    replication: Some(google_secretmanager1::api::Replication {
                        automatic: Some(google_secretmanager1::api::Automatic::default()),
                        ..Default::default()
                    }),
                    ..Default::default()
                };

                let (_, created_secret) = self
                    .hub
                    .projects()
                    .secrets_create(secret, &parent)
                    .secret_id(name)
                    .doit()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create secret in GCP: {}", e))?;

                let secret_name = created_secret
                    .name
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Created secret has no name"))?;

                let payload = AddSecretVersionRequest {
                    payload: Some(google_secretmanager1::api::SecretPayload {
                        data: Some(value.as_bytes().to_vec()),
                        ..Default::default()
                    }),
                };

                self.hub
                    .projects()
                    .secrets_add_version(payload, secret_name)
                    .doit()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to add initial secret version in GCP: {}", e))?;
            }
        }

        Ok(())
    }

    async fn get_secret(&self, name: &str) -> Result<String> {
        let parent = format!("projects/{}", self.project_id);
        let secret_path = format!("{}/secrets/{}/versions/latest", parent, name);

        let (_, response) = self
            .hub
            .projects()
            .secrets_versions_access(&secret_path)
            .doit()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get secret from GCP: {}", e))?;

        let payload = response
            .payload
            .ok_or_else(|| anyhow::anyhow!("No payload in GCP secret response"))?;

        let data = payload
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in GCP secret payload"))?;

        String::from_utf8(data).context("Invalid UTF-8 in GCP secret value")
    }

    async fn trigger_refresh(&self, service: Option<&str>) -> Result<()> {
        if let Some(svc) = service {
            println!("ℹ️  Would trigger refresh for GCP service: {}", svc);
            println!("   (e.g., Cloud Run revision, Cloud Functions update)");
            println!("   Note: Automatic refresh not implemented - manually restart your service");
        }

        Ok(())
    }
}
