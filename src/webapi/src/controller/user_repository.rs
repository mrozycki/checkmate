use crate::{
    models::{session_token::SessionToken, user::UserProfile},
    routes::user::{CreateUserRequest, LoginUserRequest},
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
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
    #[error("session not found")]
    SessionNotFound,
    #[error("user with given id not found")]
    UserNotFound,
    #[error("internal error")]
    InternalError,
}

#[tracing::instrument(name = "Saving new user in the database", skip(pool, user))]
pub async fn insert_user(
    pool: &PgPool,
    user: &CreateUserRequest,
) -> Result<(), UserRepositoryError> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(user.password.expose_secret().as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Hashing error: {}", e);
            UserRepositoryError::InternalError
        })?
        .to_string();

    sqlx::query!(
        r#"
    INSERT INTO users (id, username, password, created_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        user.username,
        password_hash,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_e) if db_e.is_unique_violation() => {
            UserRepositoryError::UserAlreadyExists {
                username: user.username.clone(),
            }
        }
        _ => {
            tracing::error!("Failed to create user in database: {:?}", e);
            UserRepositoryError::InternalError
        }
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
            tracing::error!("Failed to fetch user from database: {:?}", e);
            UserRepositoryError::InternalError
        }
    })?;

    let parsed_hash = PasswordHash::new(&data.password).map_err(|e| {
        tracing::error!("Hashing error: {}", e);
        UserRepositoryError::InternalError
    })?;

    match Argon2::default().verify_password(user.password.expose_secret().as_bytes(), &parsed_hash)
    {
        Ok(_) => Ok(data.id),
        Err(argon2::password_hash::Error::Password) => {
            Err(UserRepositoryError::InvalidUserOrPassword)
        }
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            Err(UserRepositoryError::InternalError)
        }
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
        tracing::error!("Failed to create token in database: {:?}", e);
        UserRepositoryError::InternalError
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

pub async fn get_user_id_by_token(
    pg_pool: &PgPool,
    token: SessionToken,
) -> Result<Uuid, UserRepositoryError> {
    let result = sqlx::query!(
        r#"
    SELECT user_id, valid_until from sessions 
    WHERE token = $1
            "#,
        token.to_database_value().expose_secret().to_owned()
    )
    .fetch_one(pg_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => UserRepositoryError::SessionNotFound,
        _ => {
            tracing::error!("Failed to fetch session from database: {:?}", e);
            UserRepositoryError::InternalError
        }
    })?;

    if result.valid_until > chrono::Utc::now() {
        Ok(result.user_id)
    } else {
        let _ = delete_session_by_token(pg_pool, token).await;
        Err(UserRepositoryError::SessionNotFound)
    }
}

pub async fn delete_session_by_token(
    pg_pool: &PgPool,
    token: SessionToken,
) -> Result<(), UserRepositoryError> {
    sqlx::query!(
        r#"
    DELETE FROM sessions 
    WHERE token = $1
            "#,
        token.to_database_value().expose_secret().to_owned()
    )
    .execute(pg_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete token from database: {:?}", e);
        UserRepositoryError::InternalError
    })?;
    Ok(())
}

pub async fn get_user_by_id(
    pg_pool: &PgPool,
    user_id: Uuid,
) -> Result<UserProfile, UserRepositoryError> {
    let username = sqlx::query!(
        r#"
    SELECT username FROM users
    WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pg_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => UserRepositoryError::UserNotFound,
        e => {
            tracing::error!("Failed to fetch user from database: {:?}", e);
            UserRepositoryError::InternalError
        }
    })?
    .username;

    Ok(UserProfile { username })
}
