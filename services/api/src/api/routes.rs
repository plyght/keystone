use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    api::handlers::{api_keys, credentials, members, providers, workspaces},
    auth::middleware::auth_middleware,
    credentials::{cache::CredentialCache, resolver::CredentialResolver},
    metering::MeteringService,
    supabase::SupabaseClient,
    vault::{encryption::VaultEncryption, storage::VaultStorage},
};

#[derive(Clone)]
pub struct AppState {
    pub client: SupabaseClient,
    pub vault: Arc<VaultStorage>,
    pub resolver: Arc<tokio::sync::Mutex<CredentialResolver>>,
    pub metering: Arc<MeteringService>,
}

pub fn create_router(client: SupabaseClient, redis_url: String) -> Router {
    let encryption = VaultEncryption::new().expect("Failed to initialize encryption");
    let vault = Arc::new(VaultStorage::new(client.clone(), encryption));

    let cache = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { CredentialCache::new(&redis_url, 600).await })
    })
    .expect("Failed to initialize cache");

    let resolver = Arc::new(tokio::sync::Mutex::new(CredentialResolver::new(
        client.clone(),
        VaultStorage::new(
            client.clone(),
            VaultEncryption::new().expect("Failed to initialize encryption"),
        ),
        cache,
    )));

    let metering = Arc::new(MeteringService::new(client.clone()));

    let state = AppState {
        client,
        vault,
        resolver,
        metering,
    };

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_routes(state))
}

async fn health_check() -> &'static str {
    "OK"
}

fn api_routes(state: AppState) -> Router {
    Router::new()
        .route("/workspaces", post(workspaces::create_workspace))
        .route("/workspaces", get(workspaces::list_workspaces))
        .route("/workspaces/:id", get(workspaces::get_workspace))
        .route("/workspaces/:id", put(workspaces::update_workspace))
        .route("/workspaces/:id", delete(workspaces::delete_workspace))
        .route(
            "/workspaces/:id/members",
            post(members::add_workspace_member),
        )
        .route(
            "/workspaces/:id/members",
            get(members::list_workspace_members),
        )
        .route(
            "/workspaces/:id/members/:user_id",
            put(members::update_member_role),
        )
        .route(
            "/workspaces/:id/members/:user_id",
            delete(members::remove_workspace_member),
        )
        .route(
            "/workspaces/:id/providers",
            post(providers::create_provider_config),
        )
        .route(
            "/workspaces/:id/providers",
            get(providers::list_provider_configs),
        )
        .route(
            "/workspaces/:id/providers/:provider",
            get(providers::get_provider_config),
        )
        .route(
            "/workspaces/:id/providers/:provider",
            put(providers::update_provider_config),
        )
        .route(
            "/workspaces/:id/providers/:provider",
            delete(providers::delete_provider_config),
        )
        .route(
            "/workspaces/:id/credentials",
            post(credentials::store_credential),
        )
        .route(
            "/workspaces/:id/credentials/:provider/:secret_name",
            get(credentials::get_credential),
        )
        .route("/workspaces/:id/api-keys", post(api_keys::create_api_key))
        .route("/workspaces/:id/api-keys", get(api_keys::list_api_keys))
        .route(
            "/workspaces/:id/api-keys/:key_id",
            delete(api_keys::revoke_api_key),
        )
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}
