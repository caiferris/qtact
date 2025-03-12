use actix_web::{HttpResponse, Responder, get};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().finish()
}
