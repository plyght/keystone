use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub async fn update_env_file(
    secret_name: &str,
    new_value: &str,
    env_file: Option<&str>,
) -> Result<()> {
    let env_path = if let Some(path) = env_file {
        PathBuf::from(path)
    } else {
        PathBuf::from(".env")
    };
    
    if !env_path.exists() {
        anyhow::bail!(".env file not found at: {}", env_path.display());
    }
    
    let original_contents = fs::read_to_string(&env_path)
        .context("Failed to read .env file")?;
    
    let rollback_path = PathBuf::from(".keystone-rollback");
    fs::write(&rollback_path, &original_contents)?;
    
    let mut new_contents = String::new();
    let mut found = false;
    
    for line in original_contents.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with('#') || trimmed.is_empty() {
            new_contents.push_str(line);
            new_contents.push('\n');
            continue;
        }
        
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            if key == secret_name {
                new_contents.push_str(&format!("{}={}\n", secret_name, new_value));
                found = true;
                continue;
            }
        }
        
        new_contents.push_str(line);
        new_contents.push('\n');
    }
    
    if !found {
        new_contents.push_str(&format!("{}={}\n", secret_name, new_value));
    }
    
    let temp_path = env_path.with_extension("tmp");
    fs::write(&temp_path, &new_contents)?;
    fs::rename(&temp_path, &env_path)?;
    
    println!("📝 Updated {} in {}", secret_name, env_path.display());
    println!("💡 Restart your application to use the new secret");
    println!("🔙 Rollback saved to {}", rollback_path.display());
    
    Ok(())
}

