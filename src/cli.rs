use crate::pool;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "birch")]
#[command(about = "Peel. Rotate. Renew. - Secret rotation for local .env and production hosts")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        long,
        global = true,
        help = "Dry-run mode: show what would change without making changes"
    )]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Rotate {
        secret_name: Option<String>,

        #[arg(long, help = "Environment (dev/staging/prod)")]
        env: Option<String>,

        #[arg(long, help = "Service name")]
        service: Option<String>,

        #[arg(long, help = "Trigger rotation from app signal")]
        from_signal: bool,

        #[arg(long, help = "Trigger redeploy after rotation (prod only)")]
        redeploy: bool,

        #[arg(long, help = "New secret value (if not provided, will be generated)")]
        value: Option<String>,

        #[arg(long, help = "Path to .env file (dev mode only)")]
        env_file: Option<String>,
    },

    Rollback {
        secret_name: String,

        #[arg(long, help = "Environment (dev/staging/prod)")]
        env: String,

        #[arg(long, help = "Service name")]
        service: Option<String>,

        #[arg(long, help = "Trigger redeploy after rollback (prod only)")]
        redeploy: bool,
    },

    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    Audit {
        secret_name: Option<String>,

        #[arg(long, help = "Filter by environment")]
        env: Option<String>,

        #[arg(long, help = "Show last N entries")]
        last: Option<usize>,
    },

    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    Pool {
        #[command(subcommand)]
        action: PoolAction,
    },

    #[command(hide = true)]
    DaemonInternalRun {
        #[arg(long, default_value = "127.0.0.1:9123")]
        bind: String,
    },
}

#[derive(Subcommand)]
pub enum DaemonAction {
    Start {
        #[arg(long, default_value = "127.0.0.1:9123")]
        bind: String,
    },
    Stop,
    Status,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Show,
    Init,
}

#[derive(Subcommand)]
pub enum PoolAction {
    Init {
        secret_name: String,
        #[arg(long, help = "Comma-separated list of keys")]
        keys: Option<String>,
        #[arg(long, help = "Path to file with keys (one per line)")]
        from_file: Option<String>,
    },
    Add {
        secret_name: String,
        #[arg(long)]
        key: String,
    },
    List {
        secret_name: String,
    },
    Remove {
        secret_name: String,
        #[arg(long, help = "Index of key to remove")]
        index: usize,
    },
    Import {
        secret_name: String,
        #[arg(long)]
        from_file: String,
    },
    Status {
        secret_name: String,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rotate {
            secret_name,
            env,
            service,
            from_signal,
            redeploy,
            value,
            env_file,
        } => {
            crate::rotation::rotate(
                secret_name,
                env,
                service,
                from_signal,
                redeploy,
                value,
                env_file,
                cli.dry_run,
            )
            .await
        }
        Commands::Rollback {
            secret_name,
            env,
            service,
            redeploy,
        } => crate::rollback::rollback(secret_name, env, service, redeploy, cli.dry_run).await,
        Commands::Daemon { action } => match action {
            DaemonAction::Start { bind } => crate::daemon::start(&bind).await,
            DaemonAction::Stop => crate::daemon::stop().await,
            DaemonAction::Status => crate::daemon::status().await,
        },
        Commands::Audit {
            secret_name,
            env,
            last,
        } => crate::audit::show_audit(secret_name, env, last).await,
        Commands::Config { action } => match action {
            Some(ConfigAction::Show) => crate::config::show_config().await,
            Some(ConfigAction::Init) => crate::config::init_config().await,
            None => crate::config::show_config().await,
        },
        Commands::Pool { action } => match action {
            PoolAction::Init {
                secret_name,
                keys,
                from_file,
            } => pool::pool_init(secret_name, keys, from_file).await,
            PoolAction::Add { secret_name, key } => pool::pool_add(secret_name, key).await,
            PoolAction::List { secret_name } => pool::pool_list(secret_name).await,
            PoolAction::Remove { secret_name, index } => {
                pool::pool_remove(secret_name, index).await
            }
            PoolAction::Import {
                secret_name,
                from_file,
            } => pool::pool_import(secret_name, from_file).await,
            PoolAction::Status { secret_name } => pool::pool_status(secret_name).await,
        },
        Commands::DaemonInternalRun { bind } => crate::daemon::run_daemon(bind).await,
    }
}
