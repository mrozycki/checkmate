use actix_web::{post, web, HttpResponse};
use secrecy::SecretString;
use sqlx::PgPool;

use crate::controller::user_repository::insert_user;

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
    match insert_user(&pool, &request).await {
        Ok(_) => HttpResponse::Ok().json(CreateUserResponse {
            username: request.username.clone(),
        }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}