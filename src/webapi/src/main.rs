use secrecy::ExposeSecret;
use sqlx::postgres::PgPool;
use std::net::TcpListener;
use webapi::{configuration, startup, telemetry};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber =
        telemetry::get_subscriber("checkmate", tracing::log::Level::Info, std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = configuration::get_configuration("configuration.yaml")
        .expect("Failed to read configuration.");
    let connection_pool =
        PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    startup::run(listener, connection_pool)?.await
}
