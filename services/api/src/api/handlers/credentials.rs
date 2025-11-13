use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct StoreCredentialRequest {
    pub provider: String,
    pub secret_name: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct CredentialResponse {
    pub success: bool,
}

pub async fn store_credential(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<StoreCredentialRequest>,
) -> Result<Json<CredentialResponse>, StatusCode> {
    state
        .vault
        .store_credential(workspace_id, &req.provider, &req.secret_name, &req.value)
        .await
        .map_err(|e| {
            tracing::error!("Failed to store credential: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(CredentialResponse { success: true }))
}

#[derive(Debug, Serialize)]
pub struct GetCredentialResponse {
    pub value: String,
}

pub async fn get_credential(
    State(state): State<AppState>,
    Path((workspace_id, provider, secret_name)): Path<(Uuid, String, String)>,
) -> Result<Json<GetCredentialResponse>, StatusCode> {
    let mut resolver = state.resolver.lock().await;

    let value = resolver
        .resolve(&workspace_id, &provider, &secret_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to resolve credential: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(GetCredentialResponse { value }))
}
