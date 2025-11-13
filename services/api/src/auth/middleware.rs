use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
}

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    let user_id = Uuid::parse_str(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let auth_ctx = AuthContext {
        user_id,
        workspace_id: None,
    };

    request.extensions_mut().insert(auth_ctx);

    Ok(next.run(request).await)
}
