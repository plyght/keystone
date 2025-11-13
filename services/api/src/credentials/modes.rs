use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CredentialMode {
    Hosted,
    OAuth,
    Kms,
    ApiKey,
}

impl CredentialMode {
    pub fn as_str(&self) -> &str {
        match self {
            CredentialMode::Hosted => "hosted",
            CredentialMode::OAuth => "oauth",
            CredentialMode::Kms => "kms",
            CredentialMode::ApiKey => "api_key",
        }
    }
}

impl std::str::FromStr for CredentialMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hosted" => Ok(CredentialMode::Hosted),
            "oauth" => Ok(CredentialMode::OAuth),
            "kms" => Ok(CredentialMode::Kms),
            "api_key" => Ok(CredentialMode::ApiKey),
            _ => anyhow::bail!("Invalid credential mode: {}", s),
        }
    }
}
