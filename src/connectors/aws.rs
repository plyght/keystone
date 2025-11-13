use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;

pub struct AwsConnector {
    client: SecretsManagerClient,
}

impl AwsConnector {
    pub async fn new_async(_config: &crate::config::Config) -> Result<Self> {
        let aws_config = aws_config::load_from_env().await;
        let client = SecretsManagerClient::new(&aws_config);

        Ok(Self { client })
    }

    pub fn new(config: &crate::config::Config) -> Result<Self> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("No tokio runtime available"))?;
        
        rt.block_on(Self::new_async(config))
    }
}

#[async_trait]
impl crate::connectors::Connector for AwsConnector {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()> {
        match self
            .client
            .describe_secret()
            .secret_id(name)
            .send()
            .await
        {
            Ok(_) => {
                self.client
                    .put_secret_value()
                    .secret_id(name)
                    .secret_string(value)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to update secret in AWS: {}", e))?;
            }
            Err(_) => {
                self.client
                    .create_secret()
                    .name(name)
                    .secret_string(value)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create secret in AWS: {}", e))?;
            }
        }

        Ok(())
    }

    async fn get_secret(&self, name: &str) -> Result<String> {
        let response = self
            .client
            .get_secret_value()
            .secret_id(name)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get secret from AWS: {}", e))?;

        response
            .secret_string()
            .ok_or_else(|| anyhow::anyhow!("Secret value is not a string"))
            .map(|s| s.to_string())
    }

    async fn trigger_refresh(&self, service: Option<&str>) -> Result<()> {
        if let Some(svc) = service {
            println!("Note: Automatic refresh not implemented for AWS service: {}", svc);
            println!("Manually restart your service (e.g., ECS task restart, Lambda update)");
        }

        Ok(())
    }
}
