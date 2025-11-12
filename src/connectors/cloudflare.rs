use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

pub struct CloudflareConnector {
    api_token: String,
    account_id: Option<String>,
    worker_name: Option<String>,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct UpdateSecretRequest {
    name: String,
    text: String,
    r#type: String,
}

impl CloudflareConnector {
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let api_token = config
            .connector_auth
            .cloudflare_api_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLOUDFLARE_API_TOKEN not configured"))?
            .clone();

        let account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").ok();
        let worker_name = std::env::var("CLOUDFLARE_WORKER_NAME").ok();

        Ok(Self {
            api_token,
            account_id,
            worker_name,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl crate::connectors::Connector for CloudflareConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        let account_id = self
            .account_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLOUDFLARE_ACCOUNT_ID not set"))?;

        let worker_name = self
            .worker_name
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CLOUDFLARE_WORKER_NAME not set"))?;

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/workers/scripts/{}/secrets",
            account_id, worker_name
        );

        let req = UpdateSecretRequest {
            name: name.to_string(),
            text: value.to_string(),
            r#type: "secret_text".to_string(),
        };

        let response = self
            .client
            .put(&url)
            .bearer_auth(&self.api_token)
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Cloudflare API error ({}): {}", status, text);
        }

        Ok(())
    }

    async fn get_secret(&self, _name: &str) -> Result<String> {
        anyhow::bail!("Cloudflare Workers secrets cannot be read via API (they are write-only for security)")
    }

    async fn trigger_refresh(&self, _service: Option<&str>) -> Result<()> {
        println!("ℹ️  Cloudflare Workers automatically use updated secrets on next invocation");
        Ok(())
    }
}
