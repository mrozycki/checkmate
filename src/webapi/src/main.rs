use std::net::TcpListener;
use webapi::{startup, telemetry};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber =
        telemetry::get_subscriber("checkmate", tracing::log::Level::Info, std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let listener = TcpListener::bind("127.0.0.1:8081")?;
    startup::run(listener)?.await
}
