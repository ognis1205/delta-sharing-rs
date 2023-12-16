use anyhow::Result;
use sqlx::PgPool;

pub use crate::server::middlewares::jwt::Keys as JwtKeys;

pub struct Utility;

impl Utility {
    pub async fn init_postgres(_pool: &PgPool) -> Result<()> {
        tracing::info!("initializing DB");
        Ok(())
    }
}
