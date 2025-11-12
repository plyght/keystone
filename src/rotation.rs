use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use crate::pool::KeyPool;
use std::fs;

#[allow(clippy::too_many_arguments)]
pub async fn rotate(
    secret_name: Option<String>,
    env: Option<String>,
    service: Option<String>,
    _from_signal: bool,
    redeploy: bool,
    value: Option<String>,
    env_file: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let secret_name = secret_name.ok_or_else(|| anyhow::anyhow!("SECRET_NAME is required"))?;

    let env = env.ok_or_else(|| anyhow::anyhow!("--env is required"))?;

    if dry_run {
        println!("🔍 DRY RUN: No changes will be made");
    }

    let mut lock = crate::lock::Lock::new(&env, &secret_name)?;
    lock.acquire("rotate")?;

    check_cooldown(&env, &secret_name)?;

    let new_value = if let Some(v) = value {
        v
    } else if let Some(mut pool) = KeyPool::load(&secret_name)? {
        println!("🎱 Using key pool for '{}' ({})", secret_name, 
            format!("{} available, {} exhausted", 
                pool.count_available(), 
                pool.count_exhausted()));

        if let Ok(current) = get_current_secret_value(&secret_name, &env, service.as_deref()).await {
            if let Ok(()) = pool.mark_exhausted(&current) {
                println!("   ✓ Marked current key as exhausted");
            }
        }

        match pool.get_next_available() {
            Ok(next_key) => {
                pool.save()?;
                let remaining = pool.count_available();
                if remaining <= 2 {
                    println!("⚠️  Warning: Only {} key(s) remaining in pool!", remaining);
                }
                next_key
            }
            Err(e) => {
                println!("⚠️  Pool exhausted, falling back to random generation");
                println!("   Error: {}", e);
                generate_secret()?
            }
        }
    } else {
        generate_secret()?
    };

    let masked = crate::connectors::mask_secret(&new_value);
    println!("🔄 Rotating secret '{}' in env '{}'", secret_name, env);
    println!("   New value: {}", masked);

    if !dry_run {
        if env == "dev" {
            crate::dev::update_env_file(&secret_name, &new_value, env_file.as_deref()).await?;
        } else {
            crate::prod::update_production_secret(
                &secret_name,
                &new_value,
                &env,
                service.as_deref(),
                redeploy,
            )
            .await?;
        }

        record_rotation(&env, &secret_name)?;

        let logger = crate::audit::AuditLogger::new()?;
        logger.log_with_value(
            secret_name.clone(),
            env.clone(),
            service.clone(),
            crate::audit::AuditAction::Rotate,
            true,
            Some(masked),
            Some(new_value.clone()),
        )?;

        println!("✅ Secret rotated successfully");
    } else {
        println!("✅ Dry run complete (no changes made)");
    }

    Ok(())
}

fn check_cooldown(env: &str, secret_name: &str) -> Result<()> {
    let config = crate::config::Config::load()?;
    let birch_dir = crate::config::Config::birch_dir();
    let cooldown_file = birch_dir
        .join("cooldowns")
        .join(format!("{}-{}", env, secret_name));

    if !cooldown_file.exists() {
        return Ok(());
    }

    let last_rotation_str = fs::read_to_string(&cooldown_file)?;
    let last_rotation: DateTime<Utc> = last_rotation_str.parse()?;

    let now = Utc::now();
    let elapsed = now.signed_duration_since(last_rotation);
    let cooldown = Duration::seconds(config.cooldown_seconds as i64);

    if elapsed < cooldown {
        let remaining = cooldown - elapsed;
        anyhow::bail!(
            "Cooldown active: wait {}s before rotating again",
            remaining.num_seconds()
        );
    }

    Ok(())
}

fn record_rotation(env: &str, secret_name: &str) -> Result<()> {
    let birch_dir = crate::config::Config::birch_dir();
    let cooldown_dir = birch_dir.join("cooldowns");
    fs::create_dir_all(&cooldown_dir)?;

    let cooldown_file = cooldown_dir.join(format!("{}-{}", env, secret_name));
    fs::write(&cooldown_file, Utc::now().to_rfc3339())?;

    Ok(())
}

fn generate_secret() -> Result<String> {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const SECRET_LEN: usize = 32;

    let mut rng = rand::thread_rng();
    let secret: String = (0..SECRET_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    Ok(secret)
}

async fn get_current_secret_value(
    secret_name: &str,
    env: &str,
    service: Option<&str>,
) -> Result<String> {
    if env == "dev" {
        if let Some(value) = crate::dev::get_env_secret(secret_name, None)? {
            return Ok(value);
        }
        anyhow::bail!("Secret not found in .env file")
    } else {
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

        connector.get_secret(secret_name).await
    }
}
