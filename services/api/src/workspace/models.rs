use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanTier {
    Free,
    Starter,
    Pro,
    Enterprise,
}

impl PlanTier {
    pub fn rotation_limit(&self) -> Option<u32> {
        match self {
            PlanTier::Free => Some(100),
            PlanTier::Starter => Some(1000),
            PlanTier::Pro => Some(10000),
            PlanTier::Enterprise => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PlanTier::Free => "free",
            PlanTier::Starter => "starter",
            PlanTier::Pro => "pro",
            PlanTier::Enterprise => "enterprise",
        }
    }
}

impl std::str::FromStr for PlanTier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(PlanTier::Free),
            "starter" => Ok(PlanTier::Starter),
            "pro" => Ok(PlanTier::Pro),
            "enterprise" => Ok(PlanTier::Enterprise),
            _ => anyhow::bail!("Invalid plan tier: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub plan_tier: PlanTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Admin,
    Operator,
    Viewer,
    Auditor,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Owner => "owner",
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Viewer => "viewer",
            Role::Auditor => "auditor",
        }
    }
}

impl std::str::FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Role::Owner),
            "admin" => Ok(Role::Admin),
            "operator" => Ok(Role::Operator),
            "viewer" => Ok(Role::Viewer),
            "auditor" => Ok(Role::Auditor),
            _ => anyhow::bail!("Invalid role: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
    pub created_at: DateTime<Utc>,
}
