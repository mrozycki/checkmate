use crate::routes::user::CreateUserRequest;
use chrono::Utc;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(name = "Saving new user in the database", skip(pool, user))]
pub async fn insert_user(pool: &PgPool, user: &CreateUserRequest) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO users (id, username, password, created_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        user.username,
        user.password.expose_secret(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
