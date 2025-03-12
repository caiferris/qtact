use actix_web::{HttpResponse, Responder, get, web};
use qdrant_client::{Qdrant, qdrant::QueryPointsBuilder};

#[get("/vector/similarity")]
async fn similar_vector(qclient: web::Data<Qdrant>, vector: web::Json<Vec<f32>>) -> impl Responder {
    match qclient
        .query(QueryPointsBuilder::new("qtact").query(vector.into_inner()))
        .await
    {
        Ok(response) => HttpResponse::Ok().body(format!("{:#?}", response)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}
