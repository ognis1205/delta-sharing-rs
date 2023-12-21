use anyhow::Result;
use sqlx::PgPool;

pub use crate::server::utilities::token::Hasher as HmacHasher;

pub struct Utility;

impl Utility {
    pub async fn init_postgres(_pool: &PgPool) -> Result<()> {
        tracing::info!("initializing DB");
        Ok(())
    }
}
