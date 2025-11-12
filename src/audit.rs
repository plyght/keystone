use anyhow::{Context, Result};
use base64::Engine;
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng as AeadOsRng},
    ChaCha20Poly1305, Nonce,
};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub secret_name: String,
    pub env: String,
    pub service: Option<String>,
    pub action: AuditAction,
    pub success: bool,
    pub masked_secret_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_secret_value: Option<String>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditAction {
    Rotate,
    Rollback,
    Signal,
}

pub struct AuditLogger {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    log_path: PathBuf,
    cipher: ChaCha20Poly1305,
}

impl AuditLogger {
    pub fn new() -> Result<Self> {
        let config = crate::config::Config::load()?;
        let birch_dir = crate::config::Config::birch_dir();
        let signing_key_path = birch_dir.join("signing-key");
        let encryption_key_path = birch_dir.join("encryption-key");

        fs::create_dir_all(&birch_dir)?;
        fs::create_dir_all(&config.audit_log_path)?;

        let (signing_key, verifying_key) = if signing_key_path.exists() {
            let key_bytes = fs::read(&signing_key_path)?;
            let key_array: [u8; 32] = key_bytes[..32]
                .try_into()
                .context("Invalid signing key length")?;
            let signing_key = SigningKey::from_bytes(&key_array);
            let verifying_key = signing_key.verifying_key();
            (signing_key, verifying_key)
        } else {
            let mut secret_bytes = [0u8; 32];
            OsRng.fill_bytes(&mut secret_bytes);
            let signing_key = SigningKey::from_bytes(&secret_bytes);
            let verifying_key = signing_key.verifying_key();
            fs::write(&signing_key_path, signing_key.to_bytes())?;
            (signing_key, verifying_key)
        };

        let cipher = if encryption_key_path.exists() {
            let key_bytes = fs::read(&encryption_key_path)?;
            let key_array: [u8; 32] = key_bytes[..32]
                .try_into()
                .context("Invalid encryption key length")?;
            ChaCha20Poly1305::new(&key_array.into())
        } else {
            let key = ChaCha20Poly1305::generate_key(&mut AeadOsRng);
            fs::write(&encryption_key_path, key.as_slice())?;
            ChaCha20Poly1305::new(&key)
        };

        Ok(Self {
            signing_key,
            verifying_key,
            log_path: config.audit_log_path,
            cipher,
        })
    }

    pub fn log(
        &self,
        secret_name: String,
        env: String,
        service: Option<String>,
        action: AuditAction,
        success: bool,
        masked_secret_preview: Option<String>,
    ) -> Result<()> {
        self.log_with_value(
            secret_name,
            env,
            service,
            action,
            success,
            masked_secret_preview,
            None,
        )
    }

    pub fn log_with_value(
        &self,
        secret_name: String,
        env: String,
        service: Option<String>,
        action: AuditAction,
        success: bool,
        masked_secret_preview: Option<String>,
        secret_value: Option<String>,
    ) -> Result<()> {
        let actor = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let encrypted_secret_value = if let Some(value) = secret_value {
            Some(self.encrypt_secret(&value)?)
        } else {
            None
        };

        let entry = AuditEntry {
            timestamp: Utc::now(),
            actor,
            secret_name,
            env,
            service,
            action,
            success,
            masked_secret_preview,
            encrypted_secret_value,
            signature: String::new(),
        };

        let entry_json = serde_json::to_string(&entry)?;
        let signature = self.signing_key.sign(entry_json.as_bytes());

        let mut entry_with_sig = entry;
        entry_with_sig.signature = hex::encode(signature.to_bytes());

        let log_file = self
            .log_path
            .join(format!("birch-{}.log", Utc::now().format("%Y-%m-%d")));

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        writeln!(file, "{}", serde_json::to_string(&entry_with_sig)?)?;

        Ok(())
    }

    pub fn verify_entry(&self, entry: &AuditEntry) -> Result<bool> {
        let sig_bytes = hex::decode(&entry.signature)?;
        let sig_array: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;
        let signature = Signature::from_bytes(&sig_array);

        let mut entry_without_sig = entry.clone();
        entry_without_sig.signature = String::new();
        let entry_json = serde_json::to_string(&entry_without_sig)?;

        Ok(self
            .verifying_key
            .verify(entry_json.as_bytes(), &signature)
            .is_ok())
    }

    pub fn read_logs(
        &self,
        secret_name: Option<String>,
        env: Option<String>,
        last: Option<usize>,
    ) -> Result<Vec<AuditEntry>> {
        let mut entries = Vec::new();

        if !self.log_path.exists() {
            return Ok(entries);
        }

        for entry in fs::read_dir(&self.log_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("log") {
                continue;
            }

            let contents = fs::read_to_string(&path)?;
            for line in contents.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                let audit_entry: AuditEntry = serde_json::from_str(line)?;

                if let Some(ref name) = secret_name {
                    if audit_entry.secret_name != *name {
                        continue;
                    }
                }

                if let Some(ref e) = env {
                    if audit_entry.env != *e {
                        continue;
                    }
                }

                entries.push(audit_entry);
            }
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(n) = last {
            entries.truncate(n);
        }

        Ok(entries)
    }

    fn encrypt_secret(&self, secret: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, secret.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
    }

    pub fn decrypt_secret(&self, encrypted: &str) -> Result<String> {
        let combined = base64::engine::general_purpose::STANDARD
            .decode(encrypted)
            .context("Failed to decode base64")?;

        if combined.len() < 12 {
            anyhow::bail!("Invalid encrypted data: too short");
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).context("Invalid UTF-8 in decrypted secret")
    }
}

pub async fn show_audit(
    secret_name: Option<String>,
    env: Option<String>,
    last: Option<usize>,
) -> Result<()> {
    let logger = AuditLogger::new()?;
    let entries = logger.read_logs(secret_name, env, last)?;

    if entries.is_empty() {
        println!("No audit entries found");
        return Ok(());
    }

    for entry in entries {
        println!("─────────────────────────────────────");
        println!("Time: {}", entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Actor: {}", entry.actor);
        println!("Action: {:?}", entry.action);
        println!("Secret: {}", entry.secret_name);
        println!("Env: {}", entry.env);
        if let Some(ref service) = entry.service {
            println!("Service: {}", service);
        }
        println!("Success: {}", entry.success);
        if let Some(ref preview) = entry.masked_secret_preview {
            println!("Preview: {}", preview);
        }

        let verified = logger.verify_entry(&entry)?;
        println!(
            "Signature: {} ({})",
            &entry.signature[..16],
            if verified { "✓ valid" } else { "✗ invalid" }
        );
    }
    println!("─────────────────────────────────────");

    Ok(())
}
