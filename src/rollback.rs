use anyhow::Result;
use chrono::{Duration, Utc};
use dialoguer::Confirm;
use std::fs;

pub async fn rollback(
    secret_name: String,
    env: String,
    service: Option<String>,
    redeploy: bool,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("🔍 DRY RUN: No changes will be made");
    }

    let mut lock = crate::lock::Lock::new(&env, &secret_name)?;
    lock.acquire("rollback")?;

    let config = crate::config::Config::load()?;
    let rollback_window = Duration::seconds(config.rollback_window_seconds as i64);

    check_rollback_window(&env, &secret_name, rollback_window)?;

    let previous_value = get_previous_value(&secret_name, &env)?;
    let masked = crate::connectors::mask_secret(&previous_value);

    println!("🔙 Rolling back secret '{}' in env '{}'", secret_name, env);
    println!("   Previous value: {}", masked);

    if !dry_run {
        if !Confirm::new()
            .with_prompt("Confirm rollback?")
            .default(false)
            .interact()?
        {
            anyhow::bail!("Rollback cancelled by user");
        }

        if env == "dev" {
            crate::dev::update_env_file(&secret_name, &previous_value, None).await?;
        } else {
            crate::prod::update_production_secret(
                &secret_name,
                &previous_value,
                &env,
                service.as_deref(),
                redeploy,
            )
            .await?;
        }

        let logger = crate::audit::AuditLogger::new()?;
        logger.log(
            secret_name.clone(),
            env.clone(),
            service.clone(),
            crate::audit::AuditAction::Rollback,
            true,
            Some(masked),
        )?;

        println!("✅ Secret rolled back successfully");
    } else {
        println!("✅ Dry run complete (no changes made)");
    }

    Ok(())
}

fn check_rollback_window(env: &str, secret_name: &str, window: Duration) -> Result<()> {
    let birch_dir = crate::config::Config::birch_dir();
    let cooldown_file = birch_dir
        .join("cooldowns")
        .join(format!("{}-{}", env, secret_name));

    if !cooldown_file.exists() {
        anyhow::bail!("No recent rotation found for this secret");
    }

    let last_rotation_str = fs::read_to_string(&cooldown_file)?;
    let last_rotation = last_rotation_str.parse::<chrono::DateTime<Utc>>()?;

    let now = Utc::now();
    let elapsed = now.signed_duration_since(last_rotation);

    if elapsed > window {
        println!(
            "⚠️  Rollback window expired ({} ago)",
            format_duration(elapsed)
        );
        println!("   Old key may have been revoked at provider");

        if !Confirm::new()
            .with_prompt("Continue with rollback anyway?")
            .default(false)
            .interact()?
        {
            anyhow::bail!("Rollback cancelled");
        }
    }

    Ok(())
}

fn get_previous_value(secret_name: &str, env: &str) -> Result<String> {
    let logger = crate::audit::AuditLogger::new()?;
    let entries = logger.read_logs(Some(secret_name.to_string()), Some(env.to_string()), None)?;

    let rotate_entries: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e.action, crate::audit::AuditAction::Rotate) && e.success)
        .collect();

    if rotate_entries.len() < 2 {
        anyhow::bail!(
            "No previous value found in audit logs (need at least 2 successful rotations, found {})",
            rotate_entries.len()
        );
    }

    let previous_entry = rotate_entries[1];

    if let Some(ref encrypted_value) = previous_entry.encrypted_secret_value {
        logger.decrypt_secret(encrypted_value)
    } else {
        anyhow::bail!(
            "Previous rotation did not store encrypted value (audit entry from: {})",
            previous_entry.timestamp
        );
    }
}

fn format_duration(d: Duration) -> String {
    let seconds = d.num_seconds();
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else {
        format!("{}h", seconds / 3600)
    }
}
