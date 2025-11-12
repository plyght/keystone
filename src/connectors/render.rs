use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

pub struct RenderConnector {
    api_key: String,
    service_id: Option<String>,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct UpdateEnvVarRequest {
    key: String,
    value: String,
}

impl RenderConnector {
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let api_key = config
            .connector_auth
            .render_api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("RENDER_API_KEY not configured"))?
            .clone();

        let service_id = std::env::var("RENDER_SERVICE_ID").ok();

        Ok(Self {
            api_key,
            service_id,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl crate::connectors::Connector for RenderConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        let service_id = self
            .service_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("RENDER_SERVICE_ID not set"))?;

        let url = format!("https://api.render.com/v1/services/{}/env-vars", service_id);

        let req = UpdateEnvVarRequest {
            key: name.to_string(),
            value: value.to_string(),
        };

        let response = self
            .client
            .put(&url)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Render API error ({}): {}", status, text);
        }

        Ok(())
    }

    async fn get_secret(&self, name: &str) -> Result<String> {
        let service_id = self
            .service_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("RENDER_SERVICE_ID not set"))?;

        let url = format!("https://api.render.com/v1/services/{}/env-vars", service_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Render API error ({}): {}", status, text);
        }

        let json: serde_json::Value = response.json().await?;
        if let Some(envs) = json.as_array() {
            for env in envs {
                if env["key"].as_str() == Some(name) {
                    if let Some(value) = env["value"].as_str() {
                        return Ok(value.to_string());
                    }
                }
            }
        }

        anyhow::bail!("Secret '{}' not found in Render service", name)
    }

    async fn trigger_refresh(&self, _service: Option<&str>) -> Result<()> {
        let service_id = self
            .service_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("RENDER_SERVICE_ID not set"))?;

        let url = format!("https://api.render.com/v1/services/{}/deploys", service_id);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Render deployment trigger failed ({}): {}", status, text);
        }

        Ok(())
    }
}
