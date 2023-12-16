use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use jsonwebtoken::encode;
use jsonwebtoken::Header;
use utoipa::ToSchema;

use crate::config;
use crate::config::JWT_SECRET;
use crate::server::middlewares::jwt::Claims;

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

fn new_token(iss: String, sub: String, aud: Vec<String>, jti: String, exp: i64) -> Result<String> {
    let claims = Claims {
        iss,
        sub,
        aud,
        jti,
        exp,
    };
    let token = encode(&Header::default(), &claims, &JWT_SECRET.encoding)
        .context("failed to create JWT token")?;
    Ok(token)
}

#[inline]
fn new_iss() -> Result<String> {
    Ok(config::fetch::<String>("server_addr"))
}

#[inline]
fn new_sub(recipient: String) -> Result<String> {
    Ok(recipient)
}

#[inline]
fn new_aud(provider: String) -> Result<(String, Vec<String>)> {
    let endpoint = format!(
        "{}/sharing/{}",
        config::fetch::<String>("server_addr"),
        provider
    );
    Ok((endpoint.clone(), vec![endpoint]))
}

#[inline]
fn new_jti(id: String) -> Result<String> {
    Ok(id)
}

fn new_exp(ttl: i64) -> Result<(i64, DateTime<Utc>)> {
    let ttl = u64::try_from(ttl).context("failed to convert i64 ttl to u64")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("failed to create current system time")?;
    let exp = now + Duration::from_secs(ttl);
    let exp = exp.as_secs();
    let exp = i64::try_from(exp).context("failed to convert u128 expiration seconds to i64")?;
    let exp_datetime = NaiveDateTime::from_timestamp_opt(exp, 0)
        .context("faield to parse expiration seconds to datetime")?;
    let exp_datetime = DateTime::<Utc>::from_utc(exp_datetime, Utc);
    Ok((exp, exp_datetime))
}

impl Service {
    pub fn issue(id: String, provider: String, recipient: String, ttl: i64) -> Result<Profile> {
        let iss = self::new_iss().context("failed to create issuer")?;
        let sub = self::new_sub(recipient).context("failed to create subject")?;
        let (endpoint, aud) = self::new_aud(provider).context("failed to create audience")?;
        let jti = self::new_jti(id).context("failed to create subject")?;
        let (exp, exp_datetime) =
            self::new_exp(ttl).context("expiration time calculation failed")?;
        let token = self::new_token(iss, sub, aud, jti, exp).context("profile creation failed")?;
        Ok(Profile {
            share_credentials_version: VERSION,
            endpoint,
            bearer_token: token,
            expiration_time: exp_datetime.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::JWT_SECRET;
    use jsonwebtoken::decode;
    use jsonwebtoken::Validation;
    use std::thread::sleep;
    use std::time::Duration;

    //#[test]
    fn test_expired_profile() -> Result<()> {
        let two_mins = Duration::from_millis(120000);
        let profile = Service::issue(
            testutils::rand::string(10),
            testutils::rand::string(10),
            testutils::rand::string(10),
            0,
        )
        .expect("profile should be issued properly");
        sleep(two_mins);
        let Err(_) = decode::<Claims>(
            &profile.bearer_token,
            &JWT_SECRET.decoding,
            &Validation::default(),
        ) else {
            panic!("new profile should be expired");
        };
        Ok(())
    }

    #[test]
    fn test_unexpired_profile() -> Result<()> {
        let profile = Service::issue(
            testutils::rand::string(10),
            testutils::rand::string(10),
            testutils::rand::string(10),
            testutils::rand::i64(100000, 1000000),
        )
        .expect("profile should be issued properly");
        let Ok(_) = decode::<Claims>(
            &profile.bearer_token,
            &JWT_SECRET.decoding,
            &Validation::default(),
        ) else {
            panic!("new profile should not be expired");
        };
        Ok(())
    }
}
