use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_audit_log_path")]
    pub audit_log_path: PathBuf,
    
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u64,
    
    #[serde(default = "default_rollback_window_seconds")]
    pub rollback_window_seconds: u64,
    
    #[serde(default = "default_daemon_bind")]
    pub daemon_bind: String,
    
    #[serde(default)]
    pub maintenance_windows: Vec<MaintenanceWindow>,
    
    #[serde(default)]
    pub connector_auth: ConnectorAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectorAuth {
    pub vercel_token: Option<String>,
    pub netlify_auth_token: Option<String>,
    pub render_api_key: Option<String>,
    pub cloudflare_api_token: Option<String>,
    pub fly_api_token: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub aws_region: Option<String>,
    pub gcp_credentials_path: Option<String>,
    pub azure_client_id: Option<String>,
    pub azure_client_secret: Option<String>,
    pub azure_tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    pub start_hour: u32,
    pub end_hour: u32,
    pub days: Vec<String>,
}

fn default_audit_log_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".keystone")
        .join("logs")
}

fn default_cooldown_seconds() -> u64 {
    60
}

fn default_rollback_window_seconds() -> u64 {
    3600
}

fn default_daemon_bind() -> String {
    "127.0.0.1:9123".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audit_log_path: default_audit_log_path(),
            cooldown_seconds: default_cooldown_seconds(),
            rollback_window_seconds: default_rollback_window_seconds(),
            daemon_bind: default_daemon_bind(),
            maintenance_windows: Vec::new(),
            connector_auth: ConnectorAuth::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let contents = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        let mut config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;
        
        config.apply_env_overrides();
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        
        Ok(())
    }
    
    pub fn config_path() -> PathBuf {
        std::env::var("KEYSTONE_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".keystone")
                    .join("config.toml")
            })
    }
    
    pub fn keystone_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".keystone")
    }
    
    fn apply_env_overrides(&mut self) {
        if let Ok(path) = std::env::var("KEYSTONE_AUDIT_LOG_PATH") {
            self.audit_log_path = PathBuf::from(path);
        }
        
        if let Ok(val) = std::env::var("KEYSTONE_COOLDOWN_SECONDS") {
            if let Ok(seconds) = val.parse() {
                self.cooldown_seconds = seconds;
            }
        }
        
        if let Ok(val) = std::env::var("KEYSTONE_ROLLBACK_WINDOW_SECONDS") {
            if let Ok(seconds) = val.parse() {
                self.rollback_window_seconds = seconds;
            }
        }
        
        if let Ok(val) = std::env::var("KEYSTONE_DAEMON_BIND") {
            self.daemon_bind = val;
        }
        
        if let Ok(val) = std::env::var("VERCEL_TOKEN") {
            self.connector_auth.vercel_token = Some(val);
        }
        
        if let Ok(val) = std::env::var("NETLIFY_AUTH_TOKEN") {
            self.connector_auth.netlify_auth_token = Some(val);
        }
        
        if let Ok(val) = std::env::var("RENDER_API_KEY") {
            self.connector_auth.render_api_key = Some(val);
        }
        
        if let Ok(val) = std::env::var("CLOUDFLARE_API_TOKEN") {
            self.connector_auth.cloudflare_api_token = Some(val);
        }
        
        if let Ok(val) = std::env::var("FLY_API_TOKEN") {
            self.connector_auth.fly_api_token = Some(val);
        }
        
        if let Ok(val) = std::env::var("AWS_ACCESS_KEY_ID") {
            self.connector_auth.aws_access_key_id = Some(val);
        }
        
        if let Ok(val) = std::env::var("AWS_SECRET_ACCESS_KEY") {
            self.connector_auth.aws_secret_access_key = Some(val);
        }
        
        if let Ok(val) = std::env::var("AWS_REGION") {
            self.connector_auth.aws_region = Some(val);
        }
        
        if let Ok(val) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            self.connector_auth.gcp_credentials_path = Some(val);
        }
        
        if let Ok(val) = std::env::var("AZURE_CLIENT_ID") {
            self.connector_auth.azure_client_id = Some(val);
        }
        
        if let Ok(val) = std::env::var("AZURE_CLIENT_SECRET") {
            self.connector_auth.azure_client_secret = Some(val);
        }
        
        if let Ok(val) = std::env::var("AZURE_TENANT_ID") {
            self.connector_auth.azure_tenant_id = Some(val);
        }
    }
}

pub async fn show_config() -> Result<()> {
    let config = Config::load()?;
    println!("{}", toml::to_string_pretty(&config)?);
    Ok(())
}

pub async fn init_config() -> Result<()> {
    let config_path = Config::config_path();
    
    if config_path.exists() {
        anyhow::bail!("Config file already exists at: {}", config_path.display());
    }
    
    let config = Config::default();
    config.save()?;
    
    println!("Initialized config at: {}", config_path.display());
    Ok(())
}

