use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use hex;
use hmac::{Hmac, Mac};
use sha2::{Sha224, Sha256, Sha384, Sha512};

use crate::config::SECRET;

macro_rules! expect_two {
    ($iter:expr) => {{
        let mut i = $iter;
        match (i.next(), i.next(), i.next()) {
            (Some(first), Some(second), None) => (first, second),
            _ => return Err(anyhow!("failed to parse token")),
        }
    }};
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum_macros::EnumString)]
pub enum Hasher {
    #[strum(ascii_case_insensitive)]
    Sha224,
    #[strum(ascii_case_insensitive)]
    Sha256,
    #[strum(ascii_case_insensitive)]
    Sha384,
    #[strum(ascii_case_insensitive)]
    Sha512,
}

type HmacSha224 = Hmac<Sha224>;

type HmacSha256 = Hmac<Sha256>;

type HmacSha384 = Hmac<Sha384>;

type HmacSha512 = Hmac<Sha512>;

pub struct Utility;

fn new_exp(ttl: i64) -> Result<i64> {
    let ttl = u64::try_from(ttl).context("failed to convert i64 ttl to u64")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("failed to create current system time")?;
    let exp = now + Duration::from_secs(ttl);
    let exp = exp.as_secs();
    Ok(i64::try_from(exp).context("failed to convert u128 expiration seconds to i64")?)
}

impl Utility {
    pub fn sign(tid: String, ttl: i64, hasher: &Hasher) -> Result<String> {
        let exp = new_exp(ttl).context("failed to create expiry time in hexadicimal format")?;
        match hasher {
            Hasher::Sha224 => {
                let mut mac = HmacSha224::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(format!("{}.{:x}", hex::encode(&tid), &exp).as_bytes());
                let sig = mac.finalize();
                Ok(format!(
                    "{}.{:x}.{:x}",
                    hex::encode(&tid),
                    &exp,
                    &sig.into_bytes()
                ))
            }
            Hasher::Sha256 => {
                let mut mac = HmacSha256::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(format!("{}.{:x}", hex::encode(&tid), &exp).as_bytes());
                let sig = mac.finalize();
                Ok(format!(
                    "{}.{:x}.{:x}",
                    hex::encode(&tid),
                    &exp,
                    &sig.into_bytes()
                ))
            }
            Hasher::Sha384 => {
                let mut mac = HmacSha384::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(format!("{}.{:x}", hex::encode(&tid), &exp).as_bytes());
                let sig = mac.finalize();
                Ok(format!(
                    "{}.{:x}.{:x}",
                    hex::encode(&tid),
                    &exp,
                    &sig.into_bytes()
                ))
            }
            Hasher::Sha512 => {
                let mut mac = HmacSha512::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(format!("{}.{:x}", hex::encode(&tid), &exp).as_bytes());
                let sig = mac.finalize();
                Ok(format!(
                    "{}.{:x}.{:x}",
                    hex::encode(&tid),
                    &exp,
                    &sig.into_bytes()
                ))
            }
        }
    }

    pub fn verify(token: &str, hasher: &Hasher) -> Result<()> {
        let (sig, body) = expect_two!(token.rsplitn(2, '.'));
        match hasher {
            Hasher::Sha224 => {
                let mut mac = HmacSha224::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(body.as_bytes());
                let sig = hex::decode(sig).context("failed to decode HMAC signature")?;
                Ok(mac.verify_slice(&sig)?)
            }
            Hasher::Sha256 => {
                let mut mac = HmacSha256::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(body.as_bytes());
                let sig = hex::decode(sig).context("failed to decode HMAC signature")?;
                Ok(mac.verify_slice(&sig)?)
            }
            Hasher::Sha384 => {
                let mut mac = HmacSha384::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(body.as_bytes());
                let sig = hex::decode(sig).context("failed to decode HMAC signature")?;
                Ok(mac.verify_slice(&sig)?)
            }
            Hasher::Sha512 => {
                let mut mac = HmacSha512::new_from_slice(&SECRET.as_bytes())
                    .context("failed to create HMAC")?;
                mac.update(body.as_bytes());
                let sig = hex::decode(sig).context("failed to decode HMAC signature")?;
                Ok(mac.verify_slice(&sig)?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;
    use std::str::FromStr;

    fn tamper(token: String) -> Result<String> {
        let (_, res) = expect_two!(token.splitn(2, '.'));
        let (exp, sig) = expect_two!(res.splitn(2, '.'));
        Ok(format!(
            "{}.{}.{}",
            hex::encode(testutils::rand::uuid()),
            exp,
            sig
        ))
    }

    #[test]
    fn test_hasher_enum() {
        let hashers = vec!["sha224", "sha256", "sha384", "sha512"];
        let hasher = testutils::rand::choose(&hashers);
        let hasher = Hasher::from_str(hasher);
        assert!(hasher.is_ok());
        let hashers = vec!["", "apple", "orange"];
        let hasher = testutils::rand::choose(&hashers);
        let hasher = Hasher::from_str(hasher);
        assert!(hasher.is_err());
    }

    #[test]
    fn test_sign_and_verify() {
        let hashers = vec!["sha224", "sha256", "sha384", "sha512"];
        let hasher = testutils::rand::choose(&hashers);
        let hasher = Hasher::from_str(hasher).expect("hasher should be chosen properly");
        let tid = testutils::rand::uuid();
        let ttl = testutils::rand::i64(0, 3600);
        let token = Utility::sign(tid, ttl, &hasher).expect("token should be signed properly");
        let verification = Utility::verify(&token, &hasher);
        assert!(verification.is_ok());
    }

    #[test]
    fn test_sign_and_detect_tampering() {
        let hashers = vec!["sha224", "sha256", "sha384", "sha512"];
        let hasher = testutils::rand::choose(&hashers);
        let hasher = Hasher::from_str(hasher).expect("hasher should be chosen properly");
        let tid = testutils::rand::uuid();
        let ttl = testutils::rand::i64(0, 3600);
        let token = Utility::sign(tid, ttl, &hasher).expect("token should be signed properly");
        let token = tamper(token).expect("token should be tampered properly");
        let verification = Utility::verify(&token, &hasher);
        assert!(verification.is_err());
    }
}
