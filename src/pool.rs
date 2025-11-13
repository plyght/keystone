use anyhow::{Context, Result};
use base64::Engine;
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng as AeadOsRng},
    ChaCha20Poly1305, Nonce,
};
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KeyStatus {
    Active,
    Exhausted,
    Available,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolKey {
    pub encrypted_value: String,
    pub status: KeyStatus,
    pub last_used: Option<DateTime<Utc>>,
    pub rate_limit_hit: Option<DateTime<Utc>>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPool {
    pub secret_name: String,
    pub keys: Vec<PoolKey>,
    pub current_index: usize,
    pub last_rotation: Option<DateTime<Utc>>,
}

impl KeyPool {
    pub fn new(secret_name: String) -> Self {
        Self {
            secret_name,
            keys: Vec::new(),
            current_index: 0,
            last_rotation: None,
        }
    }

    pub fn load(secret_name: &str) -> Result<Option<Self>> {
        let pool_path = Self::pool_path(secret_name);
        
        if !pool_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&pool_path)
            .context("Failed to read pool file")?;
        
        let pool: KeyPool = serde_json::from_str(&contents)
            .context("Failed to parse pool file")?;
        
        Ok(Some(pool))
    }

    pub fn save(&self) -> Result<()> {
        let pool_dir = Self::pools_dir();
        fs::create_dir_all(&pool_dir)?;

        let pool_path = Self::pool_path(&self.secret_name);
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&pool_path, contents)?;

        Ok(())
    }

    pub fn get_next_available(&mut self) -> Result<String> {
        if self.keys.is_empty() {
            anyhow::bail!("No keys in pool");
        }

        let mut next_index = None;
        for (i, key) in self.keys.iter().enumerate() {
            if key.status == KeyStatus::Available {
                next_index = Some(i);
                break;
            }
        }

        if let Some(index) = next_index {
            self.current_index = index;
            self.keys[index].status = KeyStatus::Active;
            self.keys[index].last_used = Some(Utc::now());
            self.keys[index].usage_count += 1;
            self.last_rotation = Some(Utc::now());
            
            let cipher = Self::get_cipher()?;
            Self::decrypt_value(&cipher, &self.keys[index].encrypted_value)
        } else {
            anyhow::bail!("No available keys in pool - all keys exhausted");
        }
    }

    pub fn mark_exhausted(&mut self, value: &str) -> Result<()> {
        let cipher = Self::get_cipher()?;
        
        for key in &mut self.keys {
            let decrypted = Self::decrypt_value(&cipher, &key.encrypted_value)?;
            if decrypted == value {
                key.status = KeyStatus::Exhausted;
                key.rate_limit_hit = Some(Utc::now());
                return Ok(());
            }
        }

        anyhow::bail!("Key not found in pool");
    }

    pub fn add_key(&mut self, value: String) -> Result<()> {
        let cipher = Self::get_cipher()?;
        let encrypted_value = Self::encrypt_value(&cipher, &value)?;

        let pool_key = PoolKey {
            encrypted_value,
            status: KeyStatus::Available,
            last_used: None,
            rate_limit_hit: None,
            usage_count: 0,
        };

        self.keys.push(pool_key);
        Ok(())
    }

    pub fn get_current(&self) -> Result<Option<String>> {
        if self.current_index >= self.keys.len() {
            return Ok(None);
        }

        let cipher = Self::get_cipher()?;
        let decrypted = Self::decrypt_value(&cipher, &self.keys[self.current_index].encrypted_value)?;
        Ok(Some(decrypted))
    }

    pub fn list_keys(&self) -> Vec<(usize, KeyStatus, Option<DateTime<Utc>>, String)> {
        self.keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let cipher = Self::get_cipher().ok();
                let masked = if let Some(c) = cipher {
                    if let Ok(decrypted) = Self::decrypt_value(&c, &key.encrypted_value) {
                        crate::connectors::mask_secret(&decrypted)
                    } else {
                        "***[error]".to_string()
                    }
                } else {
                    "***[error]".to_string()
                };
                
                (i, key.status.clone(), key.last_used, masked)
            })
            .collect()
    }

    pub fn count_available(&self) -> usize {
        self.keys.iter().filter(|k| k.status == KeyStatus::Available).count()
    }

    pub fn count_exhausted(&self) -> usize {
        self.keys.iter().filter(|k| k.status == KeyStatus::Exhausted).count()
    }

    pub fn count_active(&self) -> usize {
        self.keys.iter().filter(|k| k.status == KeyStatus::Active).count()
    }

    fn pools_dir() -> PathBuf {
        crate::config::Config::birch_dir().join("pools")
    }

    fn pool_path(secret_name: &str) -> PathBuf {
        Self::pools_dir().join(format!("{}.json", secret_name))
    }

    fn get_cipher() -> Result<ChaCha20Poly1305> {
        let birch_dir = crate::config::Config::birch_dir();
        let encryption_key_path = birch_dir.join("encryption-key");

        if !encryption_key_path.exists() {
            fs::create_dir_all(&birch_dir)?;
            let key = ChaCha20Poly1305::generate_key(&mut AeadOsRng);
            fs::write(&encryption_key_path, key.as_slice())?;
            Ok(ChaCha20Poly1305::new(&key))
        } else {
            let key_bytes = fs::read(&encryption_key_path)?;
            let key_array: [u8; 32] = key_bytes[..32]
                .try_into()
                .context("Invalid encryption key length")?;
            Ok(ChaCha20Poly1305::new(&key_array.into()))
        }
    }

    fn encrypt_value(cipher: &ChaCha20Poly1305, value: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, value.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
    }

    fn decrypt_value(cipher: &ChaCha20Poly1305, encrypted: &str) -> Result<String> {
        let combined = base64::engine::general_purpose::STANDARD
            .decode(encrypted)
            .context("Failed to decode base64")?;

        if combined.len() < 12 {
            anyhow::bail!("Invalid encrypted data: too short");
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).context("Invalid UTF-8 in decrypted value")
    }
}

pub async fn pool_init(
    secret_name: String,
    keys: Option<String>,
    from_file: Option<String>,
) -> Result<()> {
    let pool_path = KeyPool::pool_path(&secret_name);
    if pool_path.exists() {
        anyhow::bail!("Pool for '{}' already exists", secret_name);
    }

    let mut pool = KeyPool::new(secret_name.clone());

    if let Some(keys_str) = keys {
        for key in keys_str.split(',') {
            let key = key.trim();
            if !key.is_empty() {
                pool.add_key(key.to_string())?;
            }
        }
    }

    if let Some(file_path) = from_file {
        let contents = std::fs::read_to_string(&file_path)
            .context(format!("Failed to read file: {}", file_path))?;
        for line in contents.lines() {
            let key = line.trim();
            if !key.is_empty() && !key.starts_with('#') {
                pool.add_key(key.to_string())?;
            }
        }
    }

    if pool.keys.is_empty() {
        anyhow::bail!("No keys provided. Use --keys or --from-file to specify keys");
    }

    pool.save()?;

    println!("Created pool for '{}' with {} key(s)", secret_name, pool.keys.len());
    Ok(())
}

pub async fn pool_add(secret_name: String, key: String) -> Result<()> {
    let mut pool = KeyPool::load(&secret_name)?
        .ok_or_else(|| anyhow::anyhow!("Pool for '{}' does not exist. Use 'birch pool init' first", secret_name))?;

    pool.add_key(key)?;
    pool.save()?;

    println!("Added key to pool '{}' (now {} total keys)", secret_name, pool.keys.len());
    Ok(())
}

pub async fn pool_list(secret_name: String) -> Result<()> {
    let pool = KeyPool::load(&secret_name)?
        .ok_or_else(|| anyhow::anyhow!("Pool for '{}' does not exist", secret_name))?;

    println!("Pool: {}", secret_name);
    println!("─────────────────────────────────────");

    let keys = pool.list_keys();
    for (index, status, last_used, masked_value) in keys {
        let status_str = match status {
            KeyStatus::Active => "Active",
            KeyStatus::Exhausted => "Exhausted",
            KeyStatus::Available => "Available",
        };

        print!("{}: {} {}", index, status_str, masked_value);
        if let Some(last_used_time) = last_used {
            print!(" (last used: {})", last_used_time.format("%Y-%m-%d %H:%M:%S"));
        }
        println!();
    }

    println!("─────────────────────────────────────");
    println!("Total: {} keys", pool.keys.len());
    println!("Available: {} | Active: {} | Exhausted: {}",
        pool.count_available(),
        pool.count_active(),
        pool.count_exhausted()
    );

    Ok(())
}

pub async fn pool_remove(secret_name: String, index: usize) -> Result<()> {
    let mut pool = KeyPool::load(&secret_name)?
        .ok_or_else(|| anyhow::anyhow!("Pool for '{}' does not exist", secret_name))?;

    if index >= pool.keys.len() {
        anyhow::bail!("Index {} out of range (pool has {} keys)", index, pool.keys.len());
    }

    pool.keys.remove(index);
    pool.save()?;

    println!("Removed key at index {} from pool '{}' ({} keys remaining)", 
        index, secret_name, pool.keys.len());
    Ok(())
}

pub async fn pool_import(secret_name: String, from_file: String) -> Result<()> {
    let mut pool = KeyPool::load(&secret_name)?
        .ok_or_else(|| anyhow::anyhow!("Pool for '{}' does not exist. Use 'birch pool init' first", secret_name))?;

    let contents = std::fs::read_to_string(&from_file)
        .context(format!("Failed to read file: {}", from_file))?;

    let mut count = 0;
    for line in contents.lines() {
        let key = line.trim();
        if !key.is_empty() && !key.starts_with('#') {
            pool.add_key(key.to_string())?;
            count += 1;
        }
    }

    pool.save()?;

    println!("Imported {} key(s) into pool '{}' (now {} total keys)", 
        count, secret_name, pool.keys.len());
    Ok(())
}

pub async fn pool_status(secret_name: String) -> Result<()> {
    let pool = KeyPool::load(&secret_name)?
        .ok_or_else(|| anyhow::anyhow!("Pool for '{}' does not exist", secret_name))?;

    println!("Pool: {}", secret_name);
    println!("Status: {}", if pool.count_available() > 0 { "Ready" } else { "Exhausted" });
    println!();
    println!("Total keys:      {}", pool.keys.len());
    println!("Available:       {} ({}%)", 
        pool.count_available(),
        if pool.keys.is_empty() { 0 } else { (pool.count_available() * 100) / pool.keys.len() }
    );
    println!("Active:          {}", pool.count_active());
    println!("Exhausted:       {}", pool.count_exhausted());
    println!();
    println!("Current index:   {}", pool.current_index);
    if let Ok(Some(current_key)) = pool.get_current() {
        println!("Current key:     {}", crate::connectors::mask_secret(&current_key));
    }
    if let Some(last_rotation) = pool.last_rotation {
        println!("Last rotation:   {}", last_rotation.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("Last rotation:   Never");
    }

    if pool.count_available() <= 2 && pool.count_available() > 0 {
        println!();
        println!("Warning: Only {} key(s) remaining!", pool.count_available());
    }

    Ok(())
}

