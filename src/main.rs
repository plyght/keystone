mod audit;
mod cli;
mod config;
mod connectors;
mod daemon;
mod dev;
mod lock;
mod pool;
mod prod;
mod rollback;
mod rotation;
mod saas;
mod signals;
mod tui;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "birch=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    cli::run().await
}
