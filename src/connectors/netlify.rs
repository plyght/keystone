use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

pub struct NetlifyConnector {
    token: String,
    site_id: Option<String>,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct UpdateEnvVarRequest {
    key: String,
    values: Vec<EnvValue>,
}

#[derive(Serialize)]
struct EnvValue {
    value: String,
    context: String,
}

impl NetlifyConnector {
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let token = config
            .connector_auth
            .netlify_auth_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NETLIFY_AUTH_TOKEN not configured"))?
            .clone();
        
        let site_id = std::env::var("NETLIFY_SITE_ID").ok();
        
        Ok(Self {
            token,
            site_id,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl crate::connectors::Connector for NetlifyConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        let site_id = self
            .site_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NETLIFY_SITE_ID not set"))?;
        
        let url = format!(
            "https://api.netlify.com/api/v1/accounts/{}/env/{}",
            site_id, name
        );
        
        let req = UpdateEnvVarRequest {
            key: name.to_string(),
            values: vec![EnvValue {
                value: value.to_string(),
                context: "production".to_string(),
            }],
        };
        
        let response = self
            .client
            .put(&url)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Netlify API error ({}): {}", status, text);
        }
        
        Ok(())
    }
    
    async fn get_secret(&self, _name: &str) -> Result<String> {
        anyhow::bail!("Netlify does not expose secret values via API")
    }
    
    async fn trigger_refresh(&self, _service: Option<&str>) -> Result<()> {
        let site_id = self
            .site_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NETLIFY_SITE_ID not set"))?;
        
        let url = format!(
            "https://api.netlify.com/api/v1/sites/{}/builds",
            site_id
        );
        
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Netlify build trigger failed ({}): {}", status, text);
        }
        
        Ok(())
    }
}

