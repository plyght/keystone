use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct VercelConnector {
    token: String,
    project_id: Option<String>,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct CreateSecretRequest {
    key: String,
    value: String,
    r#type: String,
    target: Vec<String>,
}

#[derive(Deserialize)]
struct Secret {
    uid: String,
    name: String,
}

impl VercelConnector {
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let token = config
            .connector_auth
            .vercel_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("VERCEL_TOKEN not configured"))?
            .clone();
        
        let project_id = std::env::var("VERCEL_PROJECT_ID").ok();
        
        Ok(Self {
            token,
            project_id,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl crate::connectors::Connector for VercelConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        let project_id = self
            .project_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("VERCEL_PROJECT_ID not set"))?;
        
        let url = format!(
            "https://api.vercel.com/v10/projects/{}/env",
            project_id
        );
        
        let req = CreateSecretRequest {
            key: name.to_string(),
            value: value.to_string(),
            r#type: "encrypted".to_string(),
            target: vec!["production".to_string()],
        };
        
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&req)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Vercel API error ({}): {}", status, text);
        }
        
        Ok(())
    }
    
    async fn get_secret(&self, _name: &str) -> Result<String> {
        let project_id = self
            .project_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("VERCEL_PROJECT_ID not set"))?;
        
        let url = format!(
            "https://api.vercel.com/v10/projects/{}/env",
            project_id
        );
        
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch secrets from Vercel");
        }
        
        anyhow::bail!("Vercel does not expose secret values via API")
    }
    
    async fn trigger_refresh(&self, _service: Option<&str>) -> Result<()> {
        let project_id = self
            .project_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("VERCEL_PROJECT_ID not set"))?;
        
        let url = format!(
            "https://api.vercel.com/v13/deployments",
        );
        
        let body = serde_json::json!({
            "name": project_id,
            "target": "production"
        });
        
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Vercel deployment trigger failed ({}): {}", status, text);
        }
        
        Ok(())
    }
}

