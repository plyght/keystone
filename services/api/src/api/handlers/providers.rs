use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::{api::routes::AppState, credentials::modes::CredentialMode};

#[derive(Debug, Deserialize)]
pub struct CreateProviderConfigRequest {
    pub provider: String,
    pub mode: String,
    pub config: JsonValue,
}

#[derive(Debug, Serialize)]
pub struct ProviderConfigResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub provider: String,
    pub mode: String,
    pub config: JsonValue,
}

pub async fn create_provider_config(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<CreateProviderConfigRequest>,
) -> Result<Json<ProviderConfigResponse>, StatusCode> {
    let mode: CredentialMode = req.mode.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let config_id = Uuid::new_v4();

    let stmt = db_client
        .prepare(
            "INSERT INTO provider_configs (id, workspace_id, provider, mode, config_jsonb)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, workspace_id, provider, mode, config_jsonb",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(
            &stmt,
            &[
                &config_id,
                &workspace_id,
                &req.provider,
                &mode.as_str(),
                &req.config,
            ],
        )
        .await
        .map_err(|_| StatusCode::CONFLICT)?;

    let response = ProviderConfigResponse {
        id: row.get(0),
        workspace_id: row.get(1),
        provider: row.get(2),
        mode: row.get(3),
        config: row.get(4),
    };

    Ok(Json(response))
}

pub async fn list_provider_configs(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<ProviderConfigResponse>>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare(
            "SELECT id, workspace_id, provider, mode, config_jsonb FROM provider_configs
             WHERE workspace_id = $1",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = db_client
        .query(&stmt, &[&workspace_id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let configs: Vec<ProviderConfigResponse> = rows
        .iter()
        .map(|row| ProviderConfigResponse {
            id: row.get(0),
            workspace_id: row.get(1),
            provider: row.get(2),
            mode: row.get(3),
            config: row.get(4),
        })
        .collect();

    Ok(Json(configs))
}

pub async fn get_provider_config(
    State(state): State<AppState>,
    Path((workspace_id, provider)): Path<(Uuid, String)>,
) -> Result<Json<ProviderConfigResponse>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare(
            "SELECT id, workspace_id, provider, mode, config_jsonb FROM provider_configs
             WHERE workspace_id = $1 AND provider = $2",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(&stmt, &[&workspace_id, &provider])
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let response = ProviderConfigResponse {
        id: row.get(0),
        workspace_id: row.get(1),
        provider: row.get(2),
        mode: row.get(3),
        config: row.get(4),
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderConfigRequest {
    pub mode: Option<String>,
    pub config: Option<JsonValue>,
}

pub async fn update_provider_config(
    State(state): State<AppState>,
    Path((workspace_id, provider)): Path<(Uuid, String)>,
    Json(req): Json<UpdateProviderConfigRequest>,
) -> Result<Json<ProviderConfigResponse>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mode_str = if let Some(mode) = req.mode {
        let _: CredentialMode = mode.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
        mode
    } else {
        let stmt = db_client
            .prepare("SELECT mode FROM provider_configs WHERE workspace_id = $1 AND provider = $2")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let row = db_client
            .query_one(&stmt, &[&workspace_id, &provider])
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

        row.get(0)
    };

    let config = req.config.unwrap_or(serde_json::json!({}));

    let stmt = db_client
        .prepare(
            "UPDATE provider_configs SET mode = $3, config_jsonb = $4, updated_at = NOW()
             WHERE workspace_id = $1 AND provider = $2
             RETURNING id, workspace_id, provider, mode, config_jsonb",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(&stmt, &[&workspace_id, &provider, &mode_str, &config])
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let response = ProviderConfigResponse {
        id: row.get(0),
        workspace_id: row.get(1),
        provider: row.get(2),
        mode: row.get(3),
        config: row.get(4),
    };

    Ok(Json(response))
}

pub async fn delete_provider_config(
    State(state): State<AppState>,
    Path((workspace_id, provider)): Path<(Uuid, String)>,
) -> Result<StatusCode, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare("DELETE FROM provider_configs WHERE workspace_id = $1 AND provider = $2")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows_affected = db_client
        .execute(&stmt, &[&workspace_id, &provider])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows_affected > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
