use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::routes::AppState,
    workspace::models::{Role, WorkspaceMember},
};

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub member: WorkspaceMember,
}

pub async fn add_workspace_member(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<MemberResponse>, StatusCode> {
    let role: Role = req.role.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare(
            "INSERT INTO workspace_members (workspace_id, user_id, role)
             VALUES ($1, $2, $3)
             RETURNING id, workspace_id, user_id, role, created_at",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(&stmt, &[&workspace_id, &req.user_id, &role.as_str()])
        .await
        .map_err(|_| StatusCode::CONFLICT)?;

    let member = WorkspaceMember {
        id: row.get(0),
        workspace_id: row.get(1),
        user_id: row.get(2),
        role: row.get::<_, String>(3).parse().unwrap(),
        created_at: row.get(4),
    };

    Ok(Json(MemberResponse { member }))
}

pub async fn list_workspace_members(
    State(state): State<AppState>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<WorkspaceMember>>, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare(
            "SELECT id, workspace_id, user_id, role, created_at FROM workspace_members
             WHERE workspace_id = $1",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = db_client
        .query(&stmt, &[&workspace_id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let members: Vec<WorkspaceMember> = rows
        .iter()
        .map(|row| WorkspaceMember {
            id: row.get(0),
            workspace_id: row.get(1),
            user_id: row.get(2),
            role: row.get::<_, String>(3).parse().unwrap(),
            created_at: row.get(4),
        })
        .collect();

    Ok(Json(members))
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub async fn update_member_role(
    State(state): State<AppState>,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<MemberResponse>, StatusCode> {
    let role: Role = req.role.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare(
            "UPDATE workspace_members SET role = $3
             WHERE workspace_id = $1 AND user_id = $2
             RETURNING id, workspace_id, user_id, role, created_at",
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = db_client
        .query_one(&stmt, &[&workspace_id, &user_id, &role.as_str()])
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let member = WorkspaceMember {
        id: row.get(0),
        workspace_id: row.get(1),
        user_id: row.get(2),
        role: row.get::<_, String>(3).parse().unwrap(),
        created_at: row.get(4),
    };

    Ok(Json(MemberResponse { member }))
}

pub async fn remove_workspace_member(
    State(state): State<AppState>,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let db_client = state
        .client
        .get_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stmt = db_client
        .prepare("DELETE FROM workspace_members WHERE workspace_id = $1 AND user_id = $2")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows_affected = db_client
        .execute(&stmt, &[&workspace_id, &user_id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows_affected > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
