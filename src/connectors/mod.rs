use anyhow::Result;
use async_trait::async_trait;

pub mod vercel;
pub mod netlify;
pub mod render;
pub mod cloudflare;
pub mod fly;
pub mod aws;
pub mod gcp;
pub mod azure;

#[async_trait]
pub trait Connector: Send + Sync {
    async fn update_secret(&self, name: &str, value: &str) -> Result<()>;
    async fn get_secret(&self, name: &str) -> Result<String>;
    async fn trigger_refresh(&self, service: Option<&str>) -> Result<()>;
}

pub fn mask_secret(secret: &str) -> String {
    if secret.len() <= 4 {
        "***".to_string()
    } else {
        format!("***{}", &secret[secret.len() - 4..])
    }
}

