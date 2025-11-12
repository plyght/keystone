use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::fs;

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
    let secret_name = secret_name
        .ok_or_else(|| anyhow::anyhow!("SECRET_NAME is required"))?;
    
    let env = env
        .ok_or_else(|| anyhow::anyhow!("--env is required"))?;
    
    if dry_run {
        println!("🔍 DRY RUN: No changes will be made");
    }
    
    let mut lock = crate::lock::Lock::new(&env, &secret_name)?;
    lock.acquire("rotate")?;
    
    check_cooldown(&env, &secret_name)?;
    
    let new_value = if let Some(v) = value {
        v
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
            ).await?;
        }
        
        record_rotation(&env, &secret_name)?;
        
        let logger = crate::audit::AuditLogger::new()?;
        logger.log(
            secret_name.clone(),
            env.clone(),
            service.clone(),
            crate::audit::AuditAction::Rotate,
            true,
            Some(masked),
        )?;
        
        println!("✅ Secret rotated successfully");
    } else {
        println!("✅ Dry run complete (no changes made)");
    }
    
    Ok(())
}

fn check_cooldown(env: &str, secret_name: &str) -> Result<()> {
    let config = crate::config::Config::load()?;
    let keystone_dir = crate::config::Config::keystone_dir();
    let cooldown_file = keystone_dir
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
    let keystone_dir = crate::config::Config::keystone_dir();
    let cooldown_dir = keystone_dir.join("cooldowns");
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

