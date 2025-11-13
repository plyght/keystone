use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use uuid::Uuid;

const API_KEY_LENGTH: usize = 32;

pub struct ApiKeyService;

impl ApiKeyService {
    pub fn generate_api_key() -> String {
        let mut rng = rand::thread_rng();
        let key_bytes: Vec<u8> = (0..API_KEY_LENGTH).map(|_| rng.gen()).collect();
        format!("sk_{}", hex::encode(key_bytes))
    }

    pub fn hash_api_key(api_key: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(api_key.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash API key: {}", e))?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_api_key(api_key: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Failed to parse hash: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(api_key.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
