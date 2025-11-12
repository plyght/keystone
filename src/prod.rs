use anyhow::Result;
use chrono::{Datelike, Timelike};
use dialoguer::Confirm;

pub async fn update_production_secret(
    secret_name: &str,
    new_value: &str,
    env: &str,
    service: Option<&str>,
    redeploy: bool,
) -> Result<()> {
    let config = crate::config::Config::load()?;
    
    if !check_maintenance_window(&config)? {
        println!("⚠️  Outside maintenance window");
        if !Confirm::new()
            .with_prompt("Continue anyway?")
            .default(false)
            .interact()?
        {
            anyhow::bail!("Aborted by user");
        }
    }
    
    let masked = crate::connectors::mask_secret(new_value);
    println!("Preview: New secret value: {}", masked);
    
    if !Confirm::new()
        .with_prompt(format!(
            "Update '{}' in {} environment{}?",
            secret_name,
            env,
            service.map(|s| format!(" for service '{}'", s)).unwrap_or_default()
        ))
        .default(false)
        .interact()?
    {
        anyhow::bail!("Aborted by user");
    }
    
    let connector = get_connector(service)?;
    
    connector.update_secret(secret_name, new_value).await?;
    println!("✅ Secret updated");
    
    if redeploy {
        println!("🚀 Triggering redeploy...");
        connector.trigger_refresh(service).await?;
        println!("✅ Redeploy triggered");
    } else {
        println!("💡 Use --redeploy to trigger automatic redeploy");
    }
    
    Ok(())
}

fn check_maintenance_window(config: &crate::config::Config) -> Result<bool> {
    if config.maintenance_windows.is_empty() {
        return Ok(true);
    }
    
    let now = chrono::Utc::now();
    let weekday = now.weekday().to_string();
    let hour = now.hour();
    
    for window in &config.maintenance_windows {
        if window.days.iter().any(|d| d.to_lowercase() == weekday.to_lowercase()) {
            if hour >= window.start_hour && hour < window.end_hour {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

fn get_connector(service: Option<&str>) -> Result<Box<dyn crate::connectors::Connector>> {
    let config = crate::config::Config::load()?;
    
    let service_name = service.ok_or_else(|| anyhow::anyhow!("--service is required for production"))?;
    
    let connector: Box<dyn crate::connectors::Connector> = match service_name.to_lowercase().as_str() {
        "vercel" => Box::new(crate::connectors::vercel::VercelConnector::new(&config)?),
        "netlify" => Box::new(crate::connectors::netlify::NetlifyConnector::new(&config)?),
        "render" => Box::new(crate::connectors::render::RenderConnector::new(&config)?),
        "cloudflare" => Box::new(crate::connectors::cloudflare::CloudflareConnector::new(&config)?),
        "fly" => Box::new(crate::connectors::fly::FlyConnector::new(&config)?),
        "aws" => Box::new(crate::connectors::aws::AwsConnector::new(&config)?),
        "gcp" => Box::new(crate::connectors::gcp::GcpConnector::new(&config)?),
        "azure" => Box::new(crate::connectors::azure::AzureConnector::new(&config)?),
        _ => anyhow::bail!("Unknown service: {}", service_name),
    };
    
    Ok(connector)
}

