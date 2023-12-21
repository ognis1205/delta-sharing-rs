use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use sqlx::postgres::PgQueryResult;
use uuid::Uuid;

use crate::server::entities::token::Entity;
use crate::server::utilities::postgres::PgAcquire;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Row {
    pub id: Uuid,
    pub value: String,
    pub active: bool,
    pub created_by: Uuid,
    pub created_for: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Repository;

impl Repository {
    pub async fn upsert(token: &Entity, executor: impl PgAcquire<'_>) -> Result<PgQueryResult> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        sqlx::query(
            r#"INSERT INTO token (
                   id,
                   "value",
                   active,
                   created_by,
                   created_for
               ) VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT(id)
               DO UPDATE
               SET "value" = $2,
                   active = $3,
                   created_by = $4,
                   created_for = $5"#,
        )
        .bind(token.id())
        .bind(token.value())
        .bind(token.active())
        .bind(token.created_by())
        .bind(token.created_for())
        .execute(&mut *conn)
        .await
        .context(format!(
            r#"failed to upsert "{}" into [token]"#,
            token.id().as_uuid()
        ))
    }
}
