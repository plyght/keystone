use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{api::routes::AppState, auth::api_keys::ApiKeyService};

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_api_key(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, StatusCode> {
    let api_key = ApiKeyService::generate_api_key();
    let key_hash =
        ApiKeyService::hash_api_key(&api_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let key_id = Uuid::new_v4();

    let stmt = db_client
        .prepare(
            "INSERT INTO api_keys (id, workspace_id, name, key_hash)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, created_at",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(&stmt, &[&key_id, &workspace_id, &req.name, &key_hash])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = CreateApiKeyResponse {
        id: row.get(0),
        name: row.get(1),
        api_key,
        created_at: row.get(2),
    };

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub name: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn list_api_keys(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<ApiKeyInfo>>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare(
            "SELECT id, name, last_used_at, created_at, revoked_at FROM api_keys
             WHERE workspace_id = $1",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = db_client
        .query(&stmt, &[&workspace_id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let keys: Vec<ApiKeyInfo> = rows
        .iter()
        .map(|row| ApiKeyInfo {
            id: row.get(0),
            name: row.get(1),
            last_used_at: row.get(2),
            created_at: row.get(3),
            revoked_at: row.get(4),
        })
        .collect();

    Ok(Json(keys))
}

pub async fn revoke_api_key(
    State(state): State<AppState>,
    Path((workspace_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare(
            "UPDATE api_keys SET revoked_at = NOW()
             WHERE id = $1 AND workspace_id = $2",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows_affected = db_client
        .execute(&stmt, &[&key_id, &workspace_id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows_affected > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
