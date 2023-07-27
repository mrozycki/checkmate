use once_cell::sync::Lazy;
use std::{net::TcpListener, str::FromStr};
use tracing::log::Level;
use webapi::{startup, telemetry};

// Ensure that the `tracing` stack is only initialized once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test";
    if let Ok(env_var) = std::env::var("TEST_LOG") {
        let level = Level::from_str(&env_var).unwrap_or(Level::Info);
        let subscriber = telemetry::get_subscriber(subscriber_name, level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_subscriber(subscriber_name, Level::Info, std::io::sink);
        telemetry::init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
}

async fn spawn_app() -> TestApp {
    // The first time `spawn_app` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let address = format!("http://{}", listener.local_addr().unwrap());

    let server = startup::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { address }
}

#[tokio::test]
async fn ping_works() {
    // Arrange
    let app = spawn_app().await;
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
