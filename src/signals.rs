use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use crate::pool::KeyPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct RotateSignal {
    secret_name: String,
    env: String,
    service: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RollbackSignal {
    secret_name: String,
    env: String,
    service: Option<String>,
    #[serde(default)]
    redeploy: bool,
}

#[derive(Debug, Serialize)]
pub struct RotateResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pool_status: Option<PoolStatus>,
}

#[derive(Debug, Serialize)]
pub struct PoolStatus {
    total_keys: usize,
    available_keys: usize,
    exhausted_keys: usize,
    current_index: usize,
}

struct AppState {
    last_signals: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
}

pub async fn start_server(bind: &str) -> Result<()> {
    let state = AppState {
        last_signals: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/rotate", post(handle_rotate))
        .route("/rollback", post(handle_rollback))
        .route("/audit", axum::routing::get(handle_audit))
        .route("/health", axum::routing::get(handle_health))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Daemon listening on {}", bind);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_rotate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RotateSignal>,
) -> impl IntoResponse {
    let signal_key = format!("{}-{}", payload.env, payload.secret_name);

    let should_process = {
        let mut last_signals = state.last_signals.lock().await;

        if let Some(last_time) = last_signals.get(&signal_key) {
            let elapsed = Utc::now().signed_duration_since(*last_time);
            let config = match crate::config::Config::load() {
                Ok(c) => c,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RotateResponse {
                            success: false,
                            message: format!("Failed to load config: {}", e),
                            pool_status: None,
                        }),
                    );
                }
            };

            if elapsed.num_seconds() < config.cooldown_seconds as i64 {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(RotateResponse {
                        success: false,
                        message: format!(
                            "Cooldown active: {}s remaining",
                            config.cooldown_seconds as i64 - elapsed.num_seconds()
                        ),
                        pool_status: None,
                    }),
                );
            }
        }

        last_signals.insert(signal_key.clone(), Utc::now());
        true
    };

    if !should_process {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(RotateResponse {
                success: false,
                message: "Signal debounced".to_string(),
                pool_status: None,
            }),
        );
    }

    let logger = match crate::audit::AuditLogger::new() {
        Ok(l) => l,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RotateResponse {
                    success: false,
                    message: format!("Failed to initialize audit logger: {}", e),
                    pool_status: None,
                }),
            );
        }
    };

    if let Err(e) = logger.log(
        payload.secret_name.clone(),
        payload.env.clone(),
        payload.service.clone(),
        crate::audit::AuditAction::Signal,
        true,
        None,
    ) {
        tracing::error!("Failed to log signal: {}", e);
    }

    let pool_status = if let Ok(Some(pool)) = KeyPool::load(&payload.secret_name) {
        Some(PoolStatus {
            total_keys: pool.keys.len(),
            available_keys: pool.count_available(),
            exhausted_keys: pool.count_exhausted(),
            current_index: pool.current_index,
        })
    } else {
        None
    };

    tokio::spawn(async move {
        let result = crate::rotation::rotate(
            Some(payload.secret_name),
            Some(payload.env),
            payload.service,
            true,
            false,
            None,
            None,
            false,
        )
        .await;

        if let Err(e) = result {
            tracing::error!("App-signal rotation failed: {}", e);
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(RotateResponse {
            success: true,
            message: "Rotation queued".to_string(),
            pool_status,
        }),
    )
}

async fn handle_rollback(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RollbackSignal>,
) -> impl IntoResponse {
    let signal_key = format!("{}-{}-rollback", payload.env, payload.secret_name);

    let should_process = {
        let mut last_signals = state.last_signals.lock().await;

        if let Some(last_time) = last_signals.get(&signal_key) {
            let elapsed = Utc::now().signed_duration_since(*last_time);
            let config = match crate::config::Config::load() {
                Ok(c) => c,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(RotateResponse {
                            success: false,
                            message: format!("Failed to load config: {}", e),
                            pool_status: None,
                        }),
                    );
                }
            };

            if elapsed.num_seconds() < config.cooldown_seconds as i64 {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(RotateResponse {
                        success: false,
                        message: format!(
                            "Cooldown active: {}s remaining",
                            config.cooldown_seconds as i64 - elapsed.num_seconds()
                        ),
                        pool_status: None,
                    }),
                );
            }
        }

        last_signals.insert(signal_key.clone(), Utc::now());
        true
    };

    if !should_process {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(RotateResponse {
                success: false,
                message: "Signal debounced".to_string(),
                pool_status: None,
            }),
        );
    }

    let logger = match crate::audit::AuditLogger::new() {
        Ok(l) => l,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RotateResponse {
                    success: false,
                    message: format!("Failed to initialize audit logger: {}", e),
                    pool_status: None,
                }),
            );
        }
    };

    if let Err(e) = logger.log(
        payload.secret_name.clone(),
        payload.env.clone(),
        payload.service.clone(),
        crate::audit::AuditAction::Signal,
        true,
        None,
    ) {
        tracing::error!("Failed to log rollback signal: {}", e);
    }

    tokio::spawn(async move {
        let result = crate::rollback::rollback(
            payload.secret_name,
            payload.env,
            payload.service,
            payload.redeploy,
            false,
        )
        .await;

        if let Err(e) = result {
            tracing::error!("App-signal rollback failed: {}", e);
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(RotateResponse {
            success: true,
            message: "Rollback queued".to_string(),
            pool_status: None,
        }),
    )
}

async fn handle_audit(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let secret_name = params.get("secret_name").cloned();
    let env = params.get("env").cloned();
    let last = params
        .get("last")
        .and_then(|s| s.parse::<usize>().ok());

    let logger = match crate::audit::AuditLogger::new() {
        Ok(l) => l,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": format!("Failed to initialize audit logger: {}", e)
                })),
            );
        }
    };

    match logger.read_logs(secret_name, env, last) {
        Ok(entries) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "entries": entries
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to read audit logs: {}", e)
            })),
        ),
    }
}

async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
