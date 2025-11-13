use anyhow::Result;
use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct SupabaseClient {
    pool: Pool,
}

impl SupabaseClient {
    pub async fn new(database_url: &str) -> Result<Self> {
        let mut cfg = Config::new();
        cfg.url = Some(database_url.to_string());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    pub async fn get_client(&self) -> Result<deadpool_postgres::Client> {
        Ok(self.pool.get().await?)
    }
}
