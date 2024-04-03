use std::str::FromStr;

use actix_web::{
    http::{header::ContentType, StatusCode},
    web, FromRequest, HttpResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    controller::user_repository::{self, UserRepositoryError},
    models::session_token::SessionToken,
};

#[derive(Debug, Default, serde::Serialize)]
pub(crate) struct UserClaim {
    pub user_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum UserClaimError {
    #[error("invalid authorization header")]
    Unauthorized,

    #[error("internal error")]
    InternalError,
}

impl actix_web::error::ResponseError for UserClaimError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl FromRequest for UserClaim {
    type Error = UserClaimError;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|h| h.to_owned());

        let pg_pool = req.app_data::<web::Data<PgPool>>().map(|p| p.to_owned());

        Box::pin(async move {
            let token = header
                .as_ref()
                .and_then(|h| h.split_once(' '))
                .and_then(|(kind, token)| {
                    if kind.to_lowercase() == "bearer" {
                        Some(token)
                    } else {
                        None
                    }
                })
                .and_then(|token| SessionToken::from_str(token).ok())
                .ok_or(UserClaimError::Unauthorized)?;

            let Some(pg_pool) = pg_pool else {
                tracing::error!("Could not access database pool");
                return Err(UserClaimError::InternalError);
            };

            let user_id = user_repository::get_user_id_by_token(&pg_pool, token)
                .await
                .map_err(|e| match e {
                    UserRepositoryError::SessionNotFound => UserClaimError::Unauthorized,
                    _ => UserClaimError::InternalError,
                })?;

            Ok(UserClaim { user_id })
        })
    }
}
