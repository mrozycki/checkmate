mod common;

#[tokio::test]
async fn ping_works() {
    // Arrange
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("{}/ping", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(response.json::<String>().await.unwrap(), "pong");
}
