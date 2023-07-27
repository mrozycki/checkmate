use crate::routes;
use actix_web::{dev::Server, App, HttpServer};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::infra::ping)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
