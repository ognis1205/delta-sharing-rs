use anyhow::anyhow;
use axum::extract::Path;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::headers::HeaderMapExt;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;

use crate::config;
use crate::config::JWT_SECRET;
use crate::server::entities::account::Entity as AccountEntity;
use crate::server::entities::account::Name as AccountName;
use crate::server::routers::SharedState;
use crate::server::services::error::Error;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub iss: String,      // Issuer, e.g., "https://foobar.org"
    pub sub: String,      // Subject (Delta Sharing Recipient), e.g., "john"
    pub aud: Vec<String>, // Audience (Delta Sharing Provider's Tenant), e.g., ["https://foobar.org/sharing/jane"]
    pub jti: String,      // JWT ID
    pub exp: i64,         // Expiration
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[tracing::instrument(skip(next))]
pub async fn as_catalog<T>(
    Path(provider): Path<String>,
    request: Request<T>,
    next: Next<T>,
) -> std::result::Result<Response, Error>
where
    T: std::fmt::Debug,
{
    tracing::debug!("authentication/authorization must be handled in the frontend properly");
    Ok(next.run(request).await)
}

#[tracing::instrument(skip(next))]
pub async fn as_sharing<T>(
    Path(provider): Path<String>,
    request: Request<T>,
    next: Next<T>,
) -> std::result::Result<Response, Error>
where
    T: std::fmt::Debug,
{
    let Some(auth) = request.headers().typed_get::<Authorization<Bearer>>() else {
        tracing::error!("bearer token is missing");
        return Err(Error::BadRequest);
    };
    let token = auth.token().to_owned();
    let mut validation = Validation::default();
    let iss = config::fetch::<String>("server_addr");
    validation.set_issuer(&[iss]);
    let aud = format!(
        "{}/sharing/{}",
        config::fetch::<String>("server_addr"),
        provider
    );
    validation.set_audience(&[aud]);
    let Ok(token) = decode::<Claims>(&token, &JWT_SECRET.decoding, &validation) else {
        tracing::error!("bearer token validation failed");
        return Err(Error::Unauthorized)?;
    };
    let Some(state) = request.extensions().get::<SharedState>() else {
        tracing::error!(
            "request is not handled correctly due to a server error while acquiring server state"
        );
        return Err(anyhow!("failed to acquire shared state").into());
    };
    let Ok(recipient) = AccountName::new(token.claims.sub.clone()) else {
        tracing::error!("JWT claims' recipient name is malformed");
        return Err(Error::ValidationFailed);
    };
    let Ok(_) = AccountEntity::load_by_name(&recipient, &state.pg_pool).await else {
        tracing::error!(
            "request is not handled correctly due to a server error while selecting recipient"
        );
        return Err(anyhow!("error occurred while selecting recipient from database").into());
    };
    Ok(next.run(request).await)
}
