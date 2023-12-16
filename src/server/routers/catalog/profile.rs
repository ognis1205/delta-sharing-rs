use anyhow::anyhow;
use axum::extract::Extension;
use axum::extract::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use utoipa::ToSchema;

use crate::server::entities::account::Entity as AccountEntity;
use crate::server::entities::account::Name as AccountName;
use crate::server::entities::token::Entity as TokenEntity;
use crate::server::routers::SharedState;
use crate::server::services::error::Error;
use crate::server::services::profile::Profile;
use crate::server::services::profile::Service as ProfileService;
use crate::server::utilities::postgres::Utility as PostgresUtility;

#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProfilePostParams {
    pub provider: String,
}

#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProfilePostRequest {
    pub recipient: String,
    pub ttl: i64,
}

#[derive(serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProfileResponse {
    pub profile: Profile,
}

#[utoipa::path(
    post,
    path = "/catalog/:provider/profile",
    tag = "catalog",
    responses(
        (status = 200, description = "The profile were successfully returned.", body = CatalogProfileResponse),
        (status = 400, description = "The request is malformed.", body = ErrorMessage),
        (status = 401, description = "The request is unauthenticated. The bearer token is missing or incorrect.", body = ErrorMessage),
        (status = 403, description = "The request is forbidden from being fulfilled.", body = ErrorMessage),
        (status = 500, description = "The request is not handled correctly due to a server error.", body = ErrorMessage),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn post(
    Extension(state): Extension<SharedState>,
    Path(params): Path<CatalogProfilePostParams>,
    Json(payload): Json<CatalogProfilePostRequest>,
) -> Result<Response, Error> {
    let Ok(provider) = AccountName::new(params.provider) else {
        tracing::error!("requested provider data is malformed");
        return Err(Error::ValidationFailed);
    };
    let Ok(provider) = AccountEntity::load_by_name(&provider, &state.pg_pool).await else {
        tracing::error!(
            "request is not handled correctly due to a server error while selecting provider"
        );
        return Err(anyhow!("error occured while selecting provider from database").into());
    };
    let Some(provider) = provider else {
        tracing::error!("provider does not exist");
        return Err(Error::Unauthorized);
    };
    let Ok(recipient) = AccountName::new(payload.recipient) else {
        tracing::error!("requested recipient data is malformed");
        return Err(Error::ValidationFailed);
    };
    let Ok(recipient) = AccountEntity::load_by_name(&recipient, &state.pg_pool).await else {
        tracing::error!(
            "request is not handled correctly due to a server error while selecting recipient"
        );
        return Err(anyhow!("error occured while selecting recipient from database").into());
    };
    let Some(recipient) = recipient else {
        tracing::error!("recipient does not exist");
        return Err(Error::Unauthorized);
    };
    let id = uuid::Uuid::new_v4().to_string();
    let Ok(profile) = ProfileService::issue(
        id.clone(),
        provider.name().to_string(),
        recipient.name().to_string(),
        payload.ttl,
    ) else {
        tracing::error!(
            "request is not handled correctly due to a server error while creating profile"
        );
        return Err(anyhow!("failed to create profile").into());
    };
    let Ok(token) = TokenEntity::new(
        id.clone(),
        profile.bearer_token.clone(),
        true,
        provider.id().to_string(),
        recipient.id().to_string(),
    ) else {
        tracing::error!("requested profile data is malformed");
        return Err(Error::ValidationFailed);
    };
    match PostgresUtility::error(token.save(&state.pg_pool).await)? {
        Ok(_) => {
            tracing::info!("token was successfully registered");
            Ok((StatusCode::OK, Json(CatalogProfileResponse { profile })).into_response())
        }
        Err(e) if PostgresUtility::is_conflict(&e) => {
            tracing::error!("token was already registered");
            Err(Error::Conflict)
        }
        _ => {
            tracing::error!(
                "request is not handled correctly due to a server error while updating token"
            );
            Err(anyhow!("error occured while updating token").into())
        }
    }
}
