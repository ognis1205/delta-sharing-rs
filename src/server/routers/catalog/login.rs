use anyhow::anyhow;
use axum::extract::Extension;
use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use utoipa::ToSchema;

use crate::server::entities::account::Email as AccountEmail;
use crate::server::entities::account::Entity as AccountEntity;
use crate::server::entities::account::Image as AccountImage;
use crate::server::routers::SharedState;
use crate::server::services::account::Account;
use crate::server::services::account::Service as AccountService;
use crate::server::services::error::Error;
use crate::server::utilities::postgres::Utility as PostgresUtility;

#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogLoginPostRequest {
    pub email: String,
    pub image: String,
    pub social_platform: String,
    pub social_id: String,
    pub social_name: String,
}

#[derive(serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogLoginPostResponse {
    pub account: Account,
}

#[utoipa::path(
    post,
    path = "/catalog/login",
    operation_id = "CatalogLogin",
    tag = "catalog",
    request_body = CatalogLoginPostRequest,
    responses(
        (status = 200, description = "The account was successfully logged-in.", body = CatalogLoginPostResponse),
        (status = 201, description = "The account was successfully registered.", body = CatalogLoginPostResponse),
        (status = 400, description = "The request is malformed.", body = ErrorMessage),
        (status = 401, description = "The request is unauthenticated. The bearer token is missing or incorrect.", body = ErrorMessage),
        (status = 409, description = "The account was already registered.", body = ErrorMessage),
        (status = 500, description = "The request is not handled correctly due to a server error.", body = ErrorMessage),
    )
)]
#[tracing::instrument(skip(state))]
pub async fn post(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CatalogLoginPostRequest>,
) -> Result<Response, Error> {
    let Ok(email) = AccountEmail::new(payload.email.clone()) else {
        tracing::error!("requested account data is malformed");
        return Err(Error::ValidationFailed);
    };
    let Ok(image) = AccountImage::new(payload.image.clone()) else {
        tracing::error!("requested account data is malformed");
        return Err(Error::ValidationFailed);
    };
    let Ok(account) = AccountEntity::load_by_email(&email, &state.pg_pool).await else {
        tracing::error!(
            "request is not handled correctly due to a server error while selecting account"
        );
        return Err(anyhow!("failed to login").into());
    };
    let (is_new, account) = match account {
        Some(mut account) => {
            account.set_image(image);
            (false, account)
        }
        None => {
            let name = payload
                .social_name
                .split_whitespace()
                .collect::<String>()
                .to_lowercase();
            let Ok(count) = AccountService::count_by_name_prefix(&name, &state.pg_pool).await else {
                tracing::error!(
                    "request is not handled correctly due to a server error while selecting account"
                );
                return Err(anyhow!("failed to login").into());
            };
            let name = format!("{}{}", name, count.result);
            let Ok(account) = AccountEntity::new(
                None,
                name,
                payload.email,
                payload.image,
                payload.social_platform,
                payload.social_id,
                payload.social_name,
            ) else {
                tracing::error!("requested account data is malformed");
                return Err(Error::ValidationFailed);
            };
            (true, account)
        }
    };
    let status = if is_new {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    match PostgresUtility::error(account.save(&state.pg_pool).await)? {
        Ok(_) => {
            tracing::info!("account was successfully registered");
            Ok((
                status,
                Json(CatalogLoginPostResponse {
                    account: Account::from(account),
                }),
            )
                .into_response())
        }
        Err(e) if PostgresUtility::is_conflict(&e) => {
            tracing::error!("account was already registered");
            Err(Error::Conflict)
        }
        _ => {
            tracing::error!(
                "request is not handled correctly due to a server error while updating account"
            );
            Err(anyhow!("error occured while updating account").into())
        }
    }
}
