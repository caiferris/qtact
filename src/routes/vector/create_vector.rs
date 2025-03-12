use actix_web::{HttpResponse, Responder, post, web};
use qdrant_client::{
    Qdrant,
    qdrant::{PointStruct, UpsertPointsBuilder},
};

use crate::data_model::vector::Vector;

#[post("/vector")]
async fn create_vector(qclient: web::Data<Qdrant>, vector: web::Json<Vector>) -> impl Responder {
    let vector = vector.into_inner();
    let payload = vector.payload.clone().unwrap_or_default();
    let point = PointStruct {
        id: Some(vector.id.into()),
        vectors: Some(vector.vector.clone().into()),
        payload,
    };
    match qclient
        .upsert_points(UpsertPointsBuilder::new("qtact", vec![point]).wait(true))
        .await
    {
        Ok(response) => {
            dbg!("Response = {:#?}", &response);
            HttpResponse::Ok().body(format!("Vector created = {:#?}", response))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}
