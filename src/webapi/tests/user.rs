mod common;

use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, str::FromStr};
use webapi::{
    models::session_token::{SessionToken, SESSION_TOKEN_LENGTH},
    routes,
};

#[derive(Deserialize)]
struct LoginUserResponse {
    token: String,
}

#[tokio::test]
async fn creating_user_returns_a_200_for_valid_data() {
    // Arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();
    let request = json!({"username": "jozin","password": "123"});

    // Act
    let create_user_response = client
        .post(&format!("{}/user", &app.address))
        .json(&request)
        .send()
        .await
        .expect("Failed to execute request.");

    let login_user_response = client
        .post(&format!("{}/user/login", &app.address))
        .json(&request)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, create_user_response.status().as_u16());
    let parsed_create_user_response = create_user_response
        .json::<routes::user::CreateUserResponse>()
        .await
        .unwrap();
    assert_eq!(200, login_user_response.status().as_u16());
    let parsed_login_user_response = login_user_response
        .json::<LoginUserResponse>()
        .await
        .unwrap();

    assert_eq!(
        parsed_create_user_response,
        routes::user::CreateUserResponse {
            username: "jozin".to_owned()
        }
    );
    assert_eq!(
        parsed_login_user_response.token.len(),
        SESSION_TOKEN_LENGTH * 2
    );
    let token_value = SessionToken::from_str(parsed_login_user_response.token.as_str())
        .unwrap()
        .to_database_value();
    let session_data = sqlx::query!(
        r#"SELECT user_id FROM sessions
        WHERE token = $1"#,
        token_value.expose_secret()
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved token.");

    let saved = sqlx::query!("SELECT id, username FROM users",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved user.");

    assert_eq!(saved.username, "jozin");
    assert_eq!(saved.id, session_data.user_id.unwrap());
}

#[tokio::test]
async fn creating_user_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases: Vec<(HashMap<&str, &str>, &str)> = vec![
        (
            HashMap::from([("username", "jozin")]),
            "missing the password",
        ),
        (HashMap::from([("password", "123")]), "missing the username"),
        (HashMap::from([]), "missing both name and email"),
    ];

    for (invalid_req, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/user", &app.address))
            .json(&invalid_req)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn creating_user_returns_a_409_when_user_exists() {
    // Arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response1 = client
        .post(&format!("{}/user", &app.address))
        .json(&json!({
            "username": "jozin",
            "password": "123"
        }))
        .send()
        .await
        .expect("Failed to execute request.");
    let response2 = client
        .post(&format!("{}/user", &app.address))
        .json(&json!({
            "username": "jozin",
            "password": "456"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    let login_user_response = client
        .post(&format!("{}/user/login", &app.address))
        .json(&json!({
            "username": "jozin",
            "password": "123"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert

    assert_eq!(200, response1.status().as_u16());
    let json1 = response1
        .json::<routes::user::CreateUserResponse>()
        .await
        .unwrap();

    assert_eq!(
        json1,
        routes::user::CreateUserResponse {
            username: "jozin".to_owned()
        }
    );

    assert_eq!(409, response2.status().as_u16());
    let json2 = response2.json::<serde_json::Value>().await.unwrap();

    assert_eq!(json2, json!({"error": "user 'jozin' already exists"}));

    assert_eq!(200, login_user_response.status().as_u16());
    let parsed_login_user_response = login_user_response
        .json::<LoginUserResponse>()
        .await
        .unwrap();

    let token = SessionToken::from_str(parsed_login_user_response.token.as_str())
        .unwrap()
        .to_database_value();
    let sesion_data = sqlx::query!(
        r#"SELECT user_id FROM sessions
        WHERE token = $1"#,
        token.expose_secret()
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved token.");

    let saved = sqlx::query!("SELECT id, username FROM users",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved user.");

    assert_eq!(saved.username, "jozin");
    assert_eq!(saved.id, sesion_data.user_id.unwrap());
}
