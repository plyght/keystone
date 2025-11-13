use anyhow::Result;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub role: Option<String>,
}

pub struct JwtValidator {
    jwt_secret: String,
}

impl JwtValidator {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn validate_token(&self, token: &str) -> Result<Uuid> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)?;
        Ok(user_id)
    }
}
