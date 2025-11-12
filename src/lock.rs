use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct LockData {
    pid: u32,
    timestamp: DateTime<Utc>,
    operation: String,
}

pub struct Lock {
    path: PathBuf,
    acquired: bool,
}

impl Lock {
    pub fn new(env: &str, secret_name: &str) -> Result<Self> {
        let birch_dir = crate::config::Config::birch_dir();
        let locks_dir = birch_dir.join("locks");
        fs::create_dir_all(&locks_dir)?;

        let lock_file = format!("{}-{}.lock", env, secret_name);
        let path = locks_dir.join(lock_file);

        Ok(Self {
            path,
            acquired: false,
        })
    }

    pub fn acquire(&mut self, operation: &str) -> Result<()> {
        if self.path.exists() {
            let contents = fs::read_to_string(&self.path)?;
            let lock_data: LockData =
                serde_json::from_str(&contents).context("Failed to parse lock file")?;

            let now = Utc::now();
            let lock_age = now.signed_duration_since(lock_data.timestamp);
            let timeout = Duration::minutes(5);

            if lock_age < timeout {
                anyhow::bail!(
                    "Lock already held by PID {} for operation '{}' (acquired {} ago)",
                    lock_data.pid,
                    lock_data.operation,
                    format_duration(lock_age)
                );
            }

            tracing::warn!(
                "Removing stale lock from PID {} (age: {})",
                lock_data.pid,
                format_duration(lock_age)
            );
            fs::remove_file(&self.path)?;
        }

        let lock_data = LockData {
            pid: std::process::id(),
            timestamp: Utc::now(),
            operation: operation.to_string(),
        };

        fs::write(&self.path, serde_json::to_string_pretty(&lock_data)?)?;
        self.acquired = true;

        Ok(())
    }

    pub fn release(&mut self) -> Result<()> {
        if self.acquired && self.path.exists() {
            fs::remove_file(&self.path)?;
            self.acquired = false;
        }
        Ok(())
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.release();
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
