use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

pub struct FlyConnector {
    api_token: String,
    app_name: Option<String>,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct SetSecretMutation {
    query: String,
    variables: SetSecretVariables,
}

#[derive(Serialize)]
struct SetSecretVariables {
    app_name: String,
    secrets: Vec<SecretInput>,
}

#[derive(Serialize)]
struct SecretInput {
    key: String,
    value: String,
}

impl FlyConnector {
    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let api_token = config
            .connector_auth
            .fly_api_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLY_API_TOKEN not configured"))?
            .clone();

        let app_name = std::env::var("FLY_APP_NAME").ok();

        Ok(Self {
            api_token,
            app_name,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl crate::connectors::Connector for FlyConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        let app_name = self
            .app_name
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("FLY_APP_NAME not set"))?;

        let mutation = SetSecretMutation {
            query: r#"
                mutation($appName: String!, $secrets: [SecretInput!]!) {
                    setSecrets(input: {appId: $appName, secrets: $secrets}) {
                        release {
                            id
                            version
                        }
                    }
                }
            "#
            .to_string(),
            variables: SetSecretVariables {
                app_name: app_name.clone(),
                secrets: vec![SecretInput {
                    key: name.to_string(),
                    value: value.to_string(),
                }],
            },
        };

        let response = self
            .client
            .post("https://api.fly.io/graphql")
            .bearer_auth(&self.api_token)
            .json(&mutation)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Fly.io API error ({}): {}", status, text);
        }

        Ok(())
    }

    async fn get_secret(&self, _name: &str) -> Result<String> {
        anyhow::bail!("Fly.io secrets cannot be read via API (they are write-only for security)")
    }

    async fn trigger_refresh(&self, _service: Option<&str>) -> Result<()> {
        println!("ℹ️  Fly.io automatically restarts apps when secrets are updated");
        Ok(())
    }
}
