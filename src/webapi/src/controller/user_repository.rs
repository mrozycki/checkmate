use crate::routes::user::CreateUserRequest;
use chrono::Utc;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum UserRepositoryError {
    #[error("user '{username}' already exists")]
    UserAlreadyExists { username: String },
    #[error("internal server error")]
    InternalError(#[from] sqlx::Error),
}

#[tracing::instrument(name = "Saving new user in the database", skip(pool, user))]
pub async fn insert_user(
    pool: &PgPool,
    user: &CreateUserRequest,
) -> Result<(), UserRepositoryError> {
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
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_e) => {
            if db_e.is_unique_violation() {
                UserRepositoryError::UserAlreadyExists {
                    username: user.username.clone(),
                }
            } else {
                UserRepositoryError::InternalError(e)
            }
        }
        _ => UserRepositoryError::InternalError(e),
    })
    .map_err(|e| {
        if let UserRepositoryError::InternalError(_) = e {
            tracing::error!("Failed to execute query: {:?}", e);
        }
        e
    })?;
    Ok(())
}
