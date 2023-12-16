use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use sqlx::postgres::PgQueryResult;
use uuid::Uuid;

use crate::server::entities::account::Email;
use crate::server::entities::account::Entity;
use crate::server::entities::account::Name;
use crate::server::utilities::postgres::PgAcquire;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Row {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub image: String,
    pub social_platform: String,
    pub social_id: String,
    pub social_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Repository;

impl Repository {
    pub async fn upsert(account: &Entity, executor: impl PgAcquire<'_>) -> Result<PgQueryResult> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        sqlx::query(
            "INSERT INTO account (
                 id,
                 name,
                 email,
                 image,
                 social_platform,
                 social_id,
                 social_name
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT(id)
             DO UPDATE
             SET name = $2,
                 email = $3,
                 image = $4,
                 social_platform = $5,
                 social_id = $6,
                 social_name = $7",
        )
        .bind(account.id())
        .bind(account.name())
        .bind(account.email())
        .bind(account.image())
        .bind(account.social_platform())
        .bind(account.social_id())
        .bind(account.social_name())
        .execute(&mut *conn)
        .await
        .context(format!(
            r#"failed to upsert "{}" into [account]"#,
            account.id().as_uuid()
        ))
    }

    pub async fn select_by_name(name: &Name, executor: impl PgAcquire<'_>) -> Result<Option<Row>> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        let row: Option<Row> = sqlx::query_as::<_, Row>(
            "SELECT
                 id,
                 name,
                 email,
                 image,
                 social_platform,
                 social_id,
                 social_name,
                 created_at,
                 updated_at
             FROM account
             WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&mut *conn)
        .await
        .context(format!(
            r#"failed to select "{}" from [account]"#,
            name.as_str()
        ))?;
        Ok(row)
    }

    pub async fn select_by_email(
        email: &Email,
        executor: impl PgAcquire<'_>,
    ) -> Result<Option<Row>> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        let row: Option<Row> = sqlx::query_as::<_, Row>(
            "SELECT
                 id,
                 name,
                 email,
                 image,
                 social_platform,
                 social_id,
                 social_name,
                 created_at,
                 updated_at
             FROM account
             WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&mut *conn)
        .await
        .context(format!(
            r#"failed to select "{}" from [account]"#,
            email.as_str()
        ))?;
        Ok(row)
    }
}
