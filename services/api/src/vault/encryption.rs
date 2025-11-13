use anyhow::{Context, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use std::env;
use uuid::Uuid;

pub struct VaultEncryption {
    master_key: [u8; 32],
}

impl VaultEncryption {
    pub fn new() -> Result<Self> {
        let master_key_hex = env::var("VAULT_MASTER_KEY")
            .context("VAULT_MASTER_KEY environment variable not set")?;

        let master_key_bytes =
            hex::decode(&master_key_hex).context("Failed to decode VAULT_MASTER_KEY as hex")?;

        if master_key_bytes.len() != 32 {
            anyhow::bail!("VAULT_MASTER_KEY must be 32 bytes (64 hex characters)");
        }

        let mut master_key = [0u8; 32];
        master_key.copy_from_slice(&master_key_bytes);

        Ok(Self { master_key })
    }

    fn derive_workspace_key(&self, workspace_id: &Uuid) -> [u8; 32] {
        let mut key = [0u8; 32];
        let workspace_bytes = workspace_id.as_bytes();

        for i in 0..32 {
            key[i] = self.master_key[i] ^ workspace_bytes[i % 16];
        }

        key
    }

    pub fn encrypt(&self, workspace_id: &Uuid, plaintext: &str) -> Result<Vec<u8>> {
        let workspace_key = self.derive_workspace_key(workspace_id);
        let cipher = ChaCha20Poly1305::new(&workspace_key.into());

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, workspace_id: &Uuid, encrypted: &[u8]) -> Result<String> {
        if encrypted.len() < 12 {
            anyhow::bail!("Encrypted data too short");
        }

        let workspace_key = self.derive_workspace_key(workspace_id);
        let cipher = ChaCha20Poly1305::new(&workspace_key.into());

        let nonce = Nonce::from_slice(&encrypted[0..12]);
        let ciphertext = &encrypted[12..];

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed"))?;

        String::from_utf8(plaintext).context("Invalid UTF-8 in decrypted data")
    }
}

impl Default for VaultEncryption {
    fn default() -> Self {
        Self::new().expect("Failed to initialize VaultEncryption")
    }
}
