use anyhow::Result;
use getset::Getters;
use getset::Setters;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::impl_string_property;
use crate::impl_uuid_property;
use crate::server::repositories::account::Repository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    value: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Name {
    #[validate(length(min = 1))]
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Email {
    #[validate(email)]
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Image {
    #[validate(url)]
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct SocialPlatform {
    #[validate(length(min = 1))]
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct SocialId {
    #[validate(length(min = 1))]
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct SocialName {
    #[validate(length(min = 1))]
    value: String,
}

impl_uuid_property!(Id);
impl_string_property!(Name);
impl_string_property!(Email);
impl_string_property!(Image);
impl_string_property!(SocialPlatform);
impl_string_property!(SocialId);
impl_string_property!(SocialName);

#[derive(Debug, Clone, PartialEq, Eq, Getters, Setters)]
pub struct Entity {
    #[getset(get = "pub")]
    id: Id,
    #[getset(get = "pub", set = "pub")]
    name: Name,
    #[getset(get = "pub", set = "pub")]
    email: Email,
    #[getset(get = "pub", set = "pub")]
    image: Image,
    #[getset(get = "pub", set = "pub")]
    social_platform: SocialPlatform,
    #[getset(get = "pub", set = "pub")]
    social_id: SocialId,
    #[getset(get = "pub", set = "pub")]
    social_name: SocialName,
}

impl Entity {
    pub fn new(
        id: impl Into<Option<String>>,
        name: String,
        email: String,
        image: String,
        social_platform: String,
        social_id: String,
        social_name: String,
    ) -> Result<Self> {
        Ok(Self {
            id: Id::try_from(id.into().unwrap_or(uuid::Uuid::new_v4().to_string()))?,
            name: Name::new(name)?,
            email: Email::new(email)?,
            image: Image::new(image)?,
            social_platform: SocialPlatform::new(social_platform)?,
            social_id: SocialId::new(social_id)?,
            social_name: SocialName::new(social_name)?,
        })
    }

    pub async fn load_by_name(name: &Name, pg_pool: &PgPool) -> Result<Option<Self>> {
        match Repository::select_by_name(name, pg_pool).await? {
            Some(row) => Ok(Self {
                id: Id::new(row.id),
                name: Name::new(row.name)?,
                email: Email::new(row.email)?,
                image: Image::new(row.image)?,
                social_platform: SocialPlatform::new(row.social_platform)?,
                social_id: SocialId::new(row.social_id)?,
                social_name: SocialName::new(row.social_name)?,
            }
            .into()),
            _ => Ok(None),
        }
    }

    pub async fn load_by_email(email: &Email, pg_pool: &PgPool) -> Result<Option<Self>> {
        match Repository::select_by_email(email, pg_pool).await? {
            Some(row) => Ok(Self {
                id: Id::new(row.id),
                name: Name::new(row.name)?,
                email: Email::new(row.email)?,
                image: Image::new(row.image)?,
                social_platform: SocialPlatform::new(row.social_platform)?,
                social_id: SocialId::new(row.social_id)?,
                social_name: SocialName::new(row.social_name)?,
            }
            .into()),
            _ => Ok(None),
        }
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
    fn test_valid_name() {
        assert!(Name::new(testutils::rand::string(255)).is_ok());
    }

    #[test]
    fn test_invalid_name() {
        assert!(Name::new("").is_err());
    }

    #[test]
    fn test_valid_email() {
        assert!(Email::new(testutils::rand::email()).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(Email::new(testutils::rand::string(20)).is_err());
    }

    #[test]
    fn test_valid_image() {
        assert!(Image::new(testutils::rand::url()).is_ok());
    }

    #[test]
    fn test_invalid_image() {
        assert!(Image::new(testutils::rand::string(20)).is_err());
    }

    #[test]
    fn test_valid_social_platform() {
        assert!(SocialPlatform::new(testutils::rand::string(255)).is_ok());
    }

    #[test]
    fn test_invalid_social_platform() {
        assert!(SocialPlatform::new("").is_err());
    }

    #[test]
    fn test_valid_social_id() {
        assert!(SocialId::new(testutils::rand::string(255)).is_ok());
    }

    #[test]
    fn test_invalid_social_id() {
        assert!(SocialId::new("").is_err());
    }

    #[test]
    fn test_valid_social_name() {
        assert!(SocialName::new(testutils::rand::string(255)).is_ok());
    }

    #[test]
    fn test_invalid_social_name() {
        assert!(SocialName::new("").is_err());
    }
}
