use crate::impl_string_property;
use crate::impl_uuid_property;
use crate::server::entities::account::Id as AccountId;
use crate::server::entities::account::Name as AccountName;
use crate::server::repositories::share::PgRepository;
use crate::server::repositories::share::Repository;
use anyhow::Result;
use getset::Getters;
use getset::Setters;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    value: Uuid,
}

impl_uuid_property!(Id);

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Name {
    #[validate(length(min = 1))]
    value: String,
}

impl_string_property!(Name);

#[derive(Debug, Clone, PartialEq, Eq, Getters, Setters)]
pub struct Entity {
    #[getset(get = "pub")]
    id: Id,
    #[getset(get = "pub", set = "pub")]
    name: Name,
    #[getset(get = "pub")]
    created_by: AccountId,
}

impl Entity {
    pub fn new(id: impl Into<Option<String>>, name: String, created_by: String) -> Result<Self> {
        Ok(Self {
            id: Id::try_from(id.into().unwrap_or(uuid::Uuid::new_v4().to_string()))?,
            name: Name::new(name)?,
            created_by: AccountId::try_from(created_by)?,
        })
    }

    pub async fn list(
        limit: impl Into<Option<&i64>> + Send,
        offset: impl Into<Option<&i64>> + Send,
        pg_pool: &PgPool,
    ) -> Result<Vec<Self>> {
        let repo = PgRepository;
        let rows = repo.select(limit.into(), offset.into(), pg_pool).await?;
        rows.into_iter()
            .map(|row| Self::new(row.id.to_string(), row.name, row.created_by.to_string()))
            .collect()
    }

    pub async fn list_by_account_name(
        name: &AccountName,
        limit: impl Into<Option<&i64>> + Send,
        offset: impl Into<Option<&i64>> + Send,
        pg_pool: &PgPool,
    ) -> Result<Vec<Self>> {
        let repo = PgRepository;
        let rows = repo
            .select_by_account_name(name, limit.into(), offset.into(), pg_pool)
            .await?;
        rows.into_iter()
            .map(|row| Self::new(row.id.to_string(), row.name, row.created_by.to_string()))
            .collect()
    }

    pub async fn register(&self, pg_pool: &PgPool) -> Result<PgQueryResult> {
        let repo = PgRepository;
        repo.upsert(&self, pg_pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_id() {
        assert!(matches!(Id::try_from(testutils::rand::uuid()), Ok(_)));
    }

    #[test]
    fn test_invalid_id() {
        assert!(matches!(Id::try_from(testutils::rand::string(255)), Err(_)));
    }

    #[test]
    fn test_valid_name() {
        assert!(matches!(Name::new(testutils::rand::string(255)), Ok(_)));
    }

    #[test]
    fn test_invalid_name() {
        assert!(matches!(Name::new(""), Err(_)));
    }
}