use actix_web::{HttpResponse, Responder, get, web};
use qdrant_client::{Qdrant, qdrant::GetPointsBuilder};

#[get("/vector/{vector_id}")]
async fn get_vector(qclient: web::Data<Qdrant>, vector_id: web::Path<u64>) -> impl Responder {
    match qclient
        .get_points(
            GetPointsBuilder::new("qtact", vec![vector_id.into_inner().into()])
                .with_vectors(true)
                .with_payload(true),
        )
        .await
    {
        Ok(response) => {
            dbg!("Point = {:#?}", &response);
            HttpResponse::Ok().body(format!("{:#?}", response))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}
