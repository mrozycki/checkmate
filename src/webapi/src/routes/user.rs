use actix_web::{post, web, HttpResponse};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use sqlx::PgPool;

use crate::controller::user_repository::{self, UserRepositoryError};

#[derive(serde::Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: SecretString,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct CreateUserResponse {
    pub username: String,
}

#[post("/user")]
#[tracing::instrument(
    name = "Adding a new user",
    skip(request, pool),
    fields(
        username = %request.username
    )
)]
pub async fn create_user(
    request: web::Json<CreateUserRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match user_repository::insert_user(&pool, &request).await {
        Ok(_) => HttpResponse::Ok().json(CreateUserResponse {
            username: request.username.clone(),
        }),
        Err(e @ UserRepositoryError::UserAlreadyExists { .. }) => {
            HttpResponse::Conflict().json(json!({
                "error": e.to_string()
            }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(serde::Deserialize)]
pub struct LoginUserRequest {
    pub username: String,
    pub password: SecretString,
}

#[post("/user/login")]
#[tracing::instrument(
    name = "Logging in a user",
    skip(request, pool),
    fields(
        username = %request.username
    )
)]
pub async fn login_user(
    request: web::Json<LoginUserRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match user_repository::login_user(&pool, &request).await {
        Ok(token) => HttpResponse::Ok().json(json!( {
            "token": token.to_secret_string().expose_secret(),
        })),
        Err(UserRepositoryError::InvalidUserOrPassword) => HttpResponse::Unauthorized().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
