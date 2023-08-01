mod common;

use serde_json::json;
use std::collections::HashMap;
use webapi::routes;

#[tokio::test]
async fn creating_user_returns_a_200_for_valid_data() {
    // Arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(&format!("{}/user", &app.address))
        .json(&json!({"username": "jozin","password": "123"}))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
    let json = response
        .json::<routes::user::CreateUserResponse>()
        .await
        .unwrap();

    assert_eq!(
        json,
        routes::user::CreateUserResponse {
            username: "jozin".to_owned()
        }
    );

    let saved = sqlx::query!("SELECT username, password FROM users",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.username, "jozin");
    assert_eq!(saved.password, "123");
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

    let saved = sqlx::query!("SELECT username, password FROM users",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.username, "jozin");
    assert_eq!(saved.password, "123");
}
