use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{api::routes::AppState, workspace::models::Workspace};

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceResponse {
    pub workspace: Workspace,
}

pub async fn create_workspace(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let workspace_id = Uuid::new_v4();

    let stmt = db_client
        .prepare(
            "INSERT INTO workspaces (id, name, plan_tier)
             VALUES ($1, $2, 'free')
             RETURNING id, name, plan_tier, created_at, updated_at",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(&stmt, &[&workspace_id, &req.name])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let workspace = Workspace {
        id: row.get(0),
        name: row.get(1),
        plan_tier: row.get::<_, String>(2).parse().unwrap(),
        created_at: row.get(3),
        updated_at: row.get(4),
    };

    Ok(Json(WorkspaceResponse { workspace }))
}

pub async fn list_workspaces(
    State(state): State<AppState>,
) -> Result<Json<Vec<Workspace>>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare("SELECT id, name, plan_tier, created_at, updated_at FROM workspaces")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = db_client
        .query(&stmt, &[])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let workspaces: Vec<Workspace> = rows
        .iter()
        .map(|row| Workspace {
            id: row.get(0),
            name: row.get(1),
            plan_tier: row.get::<_, String>(2).parse().unwrap(),
            created_at: row.get(3),
            updated_at: row.get(4),
        })
        .collect();

    Ok(Json(workspaces))
}

pub async fn get_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkspaceResponse>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare("SELECT id, name, plan_tier, created_at, updated_at FROM workspaces WHERE id = $1")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(&stmt, &[&id])
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let workspace = Workspace {
        id: row.get(0),
        name: row.get(1),
        plan_tier: row.get::<_, String>(2).parse().unwrap(),
        created_at: row.get(3),
        updated_at: row.get(4),
    };

    Ok(Json(WorkspaceResponse { workspace }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
}

pub async fn update_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(name) = req.name {
        let stmt = db_client
            .prepare(
                "UPDATE workspaces SET name = $2, updated_at = NOW()
                 WHERE id = $1
                 RETURNING id, name, plan_tier, created_at, updated_at",
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let row = db_client
            .query_one(&stmt, &[&id, &name])
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;

        let workspace = Workspace {
            id: row.get(0),
            name: row.get(1),
            plan_tier: row.get::<_, String>(2).parse().unwrap(),
            created_at: row.get(3),
            updated_at: row.get(4),
        };

        return Ok(Json(WorkspaceResponse { workspace }));
    }

    Err(StatusCode::BAD_REQUEST)
}

pub async fn delete_workspace(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare("DELETE FROM workspaces WHERE id = $1")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows_affected = db_client
        .execute(&stmt, &[&id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows_affected > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
