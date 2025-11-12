mod cli;
mod config;
mod connectors;
mod audit;
mod lock;
mod rotation;
mod dev;
mod prod;
mod rollback;
mod daemon;
mod signals;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "keystone=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    cli::run().await
}

