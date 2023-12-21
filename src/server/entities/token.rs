use anyhow::Result;
use getset::Getters;
use getset::Setters;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::impl_bool_property;
use crate::impl_string_property;
use crate::impl_uuid_property;
use crate::server::entities::account::Id as AccountId;
use crate::server::repositories::token::Repository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    value: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Value {
    #[validate(length(min = 1))]
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Active {
    value: bool,
}

impl_uuid_property!(Id);
impl_string_property!(Value);
impl_bool_property!(Active);

#[derive(Debug, Clone, PartialEq, Eq, Getters, Setters)]
pub struct Entity {
    #[getset(get = "pub")]
    id: Id,
    #[getset(get = "pub", set = "pub")]
    value: Value,
    #[getset(get = "pub", set = "pub")]
    active: Active,
    #[getset(get = "pub")]
    created_by: AccountId,
    #[getset(get = "pub")]
    created_for: AccountId,
}

impl Entity {
    pub fn new(
        id: impl Into<Option<String>>,
        value: String,
        active: bool,
        created_by: String,
        created_for: String,
    ) -> Result<Self> {
        Ok(Self {
            id: Id::try_from(id.into().unwrap_or(uuid::Uuid::new_v4().to_string()))?,
            value: Value::new(value)?,
            active: Active::new(active),
            created_by: AccountId::try_from(created_by)?,
            created_for: AccountId::try_from(created_for)?,
        })
    }

    pub async fn save(&self, pg_pool: &PgPool) -> Result<PgQueryResult> {
        Repository::upsert(self, pg_pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_id() {
        assert!(Id::try_from(testutils::rand::uuid()).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        assert!(Id::try_from(testutils::rand::string(255)).is_err());
    }

    #[test]
    fn test_valid_value() {
        assert!(Value::new(testutils::rand::string(255)).is_ok());
    }

    #[test]
    fn test_invalid_value() {
        assert!(Value::new("").is_err());
    }
}
