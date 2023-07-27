use actix_web::{get, HttpResponse, Responder};

#[get("/ping")]
#[tracing::instrument(name = "Ping request")]
pub(crate) async fn ping() -> impl Responder {
    HttpResponse::Ok().json("pong")
}
