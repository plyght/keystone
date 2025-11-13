use anyhow::Result;
use chrono::{Date, Utc};
use uuid::Uuid;

use crate::supabase::SupabaseClient;
use crate::workspace::models::PlanTier;

pub struct MeteringService {
    client: SupabaseClient,
}

impl MeteringService {
    pub fn new(client: SupabaseClient) -> Self {
        Self { client }
    }

    pub async fn increment_rotation_count(&self, workspace_id: Uuid) -> Result<()> {
        let db_client = self.client.get_client().await?;
        let today = chrono::Utc::now().date_naive();

        let stmt = db_client
            .prepare(
                "INSERT INTO rotation_metering (workspace_id, date, rotation_count)
                 VALUES ($1, $2, 1)
                 ON CONFLICT (workspace_id, date)
                 DO UPDATE SET rotation_count = rotation_metering.rotation_count + 1",
            )
            .await?;

        db_client.execute(&stmt, &[&workspace_id, &today]).await?;

        Ok(())
    }

    pub async fn get_rotation_count(&self, workspace_id: Uuid) -> Result<u32> {
        let db_client = self.client.get_client().await?;
        let today = chrono::Utc::now().date_naive();

        let stmt = db_client
            .prepare(
                "SELECT rotation_count FROM rotation_metering
                 WHERE workspace_id = $1 AND date = $2",
            )
            .await?;

        let rows = db_client.query(&stmt, &[&workspace_id, &today]).await?;

        if rows.is_empty() {
            return Ok(0);
        }

        let count: i32 = rows[0].get(0);
        Ok(count as u32)
    }

    pub async fn check_rotation_limit(
        &self,
        workspace_id: Uuid,
        plan_tier: &PlanTier,
    ) -> Result<bool> {
        if let Some(limit) = plan_tier.rotation_limit() {
            let count = self.get_rotation_count(workspace_id).await?;
            Ok(count < limit)
        } else {
            Ok(true)
        }
    }
}
