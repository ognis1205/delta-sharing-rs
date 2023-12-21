use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use utoipa::ToSchema;

use crate::config;
use crate::config::HASHER;
use crate::server::utilities::token::Utility as TokenUtility;

pub const VERSION: i32 = 1;

#[derive(serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub share_credentials_version: i32,
    pub endpoint: String,
    pub bearer_token: String,
    pub expiration_time: String,
}

pub struct Service;

#[inline]
fn new_endpoint(provider: String) -> Result<String> {
    Ok(format!(
        "{}/sharing/{}",
        config::fetch::<String>("server_addr"),
        provider
    ))
}

#[inline]
fn new_expiration_time(ttl: i64) -> Result<DateTime<Utc>> {
    let ttl = u64::try_from(ttl).context("failed to convert i64 ttl to u64")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("failed to create current system time")?;
    let exp = now + Duration::from_secs(ttl);
    let exp = exp.as_secs();
    let exp = i64::try_from(exp).context("failed to convert u128 expiration seconds to i64")?;
    let exp = NaiveDateTime::from_timestamp_opt(exp, 0)
        .context("faield to parse expiration seconds to datetime")?;
    let exp = DateTime::<Utc>::from_utc(exp, Utc);
    Ok(exp)
}

impl Service {
    pub fn issue(id: String, provider: String, ttl: i64) -> Result<Profile> {
        let endpoint =
            new_endpoint(provider).context("failed to create profile while creating endpoint")?;
        let token = TokenUtility::sign(id, ttl, &HASHER)
            .context("faield to create profile while signing toke")?;
        let expiration_time = new_expiration_time(ttl)
            .context("failed to create profile while parsing expiration time")?;
        Ok(Profile {
            share_credentials_version: VERSION,
            endpoint,
            bearer_token: token,
            expiration_time: expiration_time.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    macro_rules! expect_two {
        ($iter:expr) => {{
            let mut i = $iter;
            match (i.next(), i.next(), i.next()) {
                (Some(first), Some(second), None) => (first, second),
                _ => return Err(anyhow!("failed to parse token")),
            }
        }};
    }

    #[test]
    fn test_profile() -> Result<()> {
        let id = testutils::rand::uuid();
        let provider = testutils::rand::string(10);
        let ttl = testutils::rand::i64(100000, 1000000);
        let profile = Service::issue(id.clone(), provider.clone(), ttl)
            .expect("profile should be issued properly");
        let (signed_id, _) = expect_two!(profile.bearer_token.splitn(2, '.'));
        assert_ne!(id, signed_id);
        Ok(())
    }
}
