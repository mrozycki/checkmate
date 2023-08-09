use crate::{
    models::session_token::SessionToken,
    routes::user::{CreateUserRequest, LoginUserRequest},
};
use chrono::Utc;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum UserRepositoryError {
    #[error("user '{username}' already exists")]
    UserAlreadyExists { username: String },
    #[error("invalid user or password")]
    InvalidUserOrPassword,
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

#[tracing::instrument(name = "Validating users password", skip(pool, user))]
async fn validate_password(
    pool: &PgPool,
    user: &LoginUserRequest,
) -> Result<Uuid, UserRepositoryError> {
    let data = sqlx::query!(
        r#"
    SELECT password, id from users
    WHERE username = $1
            "#,
        user.username
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => UserRepositoryError::InvalidUserOrPassword,
        _ => {
            tracing::error!("Failed to execute query: {:?}", e);
            UserRepositoryError::InternalError(e)
        }
    })?;
    if *user.password.expose_secret() == data.password {
        Ok(data.id)
    } else {
        Err(UserRepositoryError::InvalidUserOrPassword)
    }
}

#[tracing::instrument(name = "Creating session token", skip(pool, user_id))]
async fn create_token(pool: &PgPool, user_id: &Uuid) -> Result<SessionToken, UserRepositoryError> {
    let new_token = SessionToken::generate_new();

    sqlx::query!(
        r#"
    INSERT INTO sessions (token, user_id, valid_until)
    VALUES ($1, $2, $3)
            "#,
        new_token.to_database_value().expose_secret().to_owned(),
        user_id,
        Utc::now() + chrono::Duration::days(7)
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(new_token)
}

pub async fn login_user(
    pool: &PgPool,
    user: &LoginUserRequest,
) -> Result<SessionToken, UserRepositoryError> {
    let user_id = validate_password(pool, user).await?;
    let token = create_token(pool, &user_id).await?;
    Ok(token)
}
