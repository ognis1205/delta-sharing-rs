use anyhow::Context;
use anyhow::Result;
use sqlx::query_builder::QueryBuilder;
use sqlx::Execute;
use utoipa::ToSchema;

use crate::server::entities::account::Email as AccountEmail;
use crate::server::entities::account::Entity as AccountEntity;
use crate::server::entities::account::Name as AccountName;
use crate::server::utilities::postgres::PgAcquire;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub name: String,
    pub email: String,
    pub image: String,
    pub social_platform: String,
    pub social_id: String,
    pub social_name: String,
}

impl Account {
    pub fn from(entity: AccountEntity) -> Self {
        Self {
            name: entity.name().to_string(),
            email: entity.email().to_string(),
            image: entity.image().to_string(),
            social_platform: entity.social_platform().to_string(),
            social_id: entity.social_id().to_string(),
            social_name: entity.social_name().to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CountByNamePrefix {
    pub result: i64,
}

pub struct Service;

impl Service {
    pub async fn query(
        limit: Option<&i64>,
        after: Option<&AccountName>,
        executor: impl PgAcquire<'_>,
    ) -> Result<Vec<Account>> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        let mut builder = QueryBuilder::new(
            "SELECT
                 name,
                 email,
                 image,
                 social_platform,
                 social_id,
                 social_name
             FROM account",
        );
        if let Some(name) = after {
            builder.push(" WHERE name >= ");
            builder.push_bind(name);
        }
        builder.push(" ORDER BY name ");
        if let Some(limit) = limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit);
        }
        let mut query = sqlx::query_as::<_, Account>(builder.build().sql());
        if let Some(name) = after {
            query = query.bind(name);
        }
        if let Some(limit) = limit {
            query = query.bind(limit);
        }
        let rows: Vec<Account> = query
            .fetch_all(&mut *conn)
            .await
            .context("failed to list accounts from [account]")?;
        Ok(rows)
    }

    pub async fn query_by_name(
        name: &AccountName,
        executor: impl PgAcquire<'_>,
    ) -> Result<Option<Account>> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        let row: Option<Account> = sqlx::query_as::<_, Account>(
            "SELECT
                 name,
                 email,
                 image,
                 social_platform,
                 social_id,
                 social_name
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

    pub async fn query_by_email(
        email: &AccountEmail,
        executor: impl PgAcquire<'_>,
    ) -> Result<Option<Account>> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        let row: Option<Account> = sqlx::query_as::<_, Account>(
            "SELECT
                 name,
                 email,
                 image,
                 social_platform,
                 social_id,
                 social_name
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

    pub async fn count_by_name_prefix(
        prefix: &String,
        executor: impl PgAcquire<'_>,
    ) -> Result<CountByNamePrefix> {
        let mut conn = executor
            .acquire()
            .await
            .context("failed to acquire postgres connection")?;
        let count: CountByNamePrefix = sqlx::query_as::<_, CountByNamePrefix>(
            "SELECT
                 COUNT(1) as result
             FROM account
             WHERE name LIKE CONCAT($1, '%')",
        )
        .bind(prefix)
        .fetch_one(&mut *conn)
        .await
        .context(format!(
            r#"failed to count "{}" from [account]"#,
            prefix.as_str()
        ))?;
        Ok(count)
    }
}
