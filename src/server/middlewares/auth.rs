//use anyhow::anyhow;
use axum::extract::Path;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::headers::HeaderMapExt;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::config::HASHER;
//use crate::server::entities::account::Entity as AccountEntity;
//use crate::server::entities::account::Name as AccountName;
//use crate::server::routers::SharedState;
use crate::server::services::error::Error;
use crate::server::utilities::token::Utility as TokenUtility;

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
    let Ok(_) = TokenUtility::verify(&token, &HASHER) else {
        tracing::error!("bearer token validation failed");
        return Err(Error::Unauthorized)?;
    };
    // NOTE:
    // The following lines commented out
    //
    //    let Some(state) = request.extensions().get::<SharedState>() else {
    //        tracing::error!(
    //            "request is not handled correctly due to a server error while acquiring server state"
    //        );
    //        return Err(anyhow!("failed to acquire shared state").into());
    //    };
    //    let Ok(recipient) = AccountName::new(token.claims.sub.clone()) else {
    //        tracing::error!("JWT claims' recipient name is malformed");
    //        return Err(Error::ValidationFailed);
    //    };
    //    let Ok(_) = AccountEntity::load_by_name(&recipient, &state.pg_pool).await else {
    //        tracing::error!(
    //            "request is not handled correctly due to a server error while selecting recipient"
    //        );
    //        return Err(anyhow!("error occurred while selecting recipient from database").into());
    //    };
    //
    Ok(next.run(request).await)
}
