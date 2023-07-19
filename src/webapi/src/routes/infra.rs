use actix_web::{get, HttpResponse, Responder};
use uuid::Uuid;

#[get("/ping")]
#[tracing::instrument(
    name = "Ping request", 
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub(crate) async fn ping() -> impl Responder {
    HttpResponse::Ok().json("pong")
}
