use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use qdrant_client::{
    Qdrant,
    qdrant::{
        DeletePointsBuilder, GetPointsBuilder, PointStruct, PointsIdsList, QueryPointsBuilder,
        UpsertPointsBuilder,
    },
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

#[put("/vector/{vector_id}")]
async fn update_vector(qclient: web::Data<Qdrant>, vector: web::Json<Vector>) -> impl Responder {
    let vector = vector.into_inner();
    let payload = vector.payload.clone().unwrap_or_default();
    let point = PointStruct {
        id: Some(vector.id.into()),
        vectors: Some(vector.vector.into()),
        payload,
    };

    match qclient
        .upsert_points(UpsertPointsBuilder::new("qtact", vec![point]))
        .await
    {
        Ok(response) => {
            dbg!("Point Updated = {:#?}", &response);
            HttpResponse::Ok().body(format!("{:#?}", response))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}

#[delete("/vector/{vector_id}")]
async fn delete_vector(qclient: web::Data<Qdrant>, vector_id: web::Path<u64>) -> impl Responder {
    match qclient
        .delete_points(
            DeletePointsBuilder::new("qtact")
                .points(PointsIdsList {
                    ids: vec![vector_id.into_inner().into()],
                })
                .wait(true),
        )
        .await
    {
        Ok(response) => {
            dbg!("Deleted vector = {:?}", &response);
            HttpResponse::Ok().body(format!("Deleted vector = {:#?}", response))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}

#[get("/matchvector")]
async fn match_vector(qclient: web::Data<Qdrant>, vector: web::Json<Vec<f32>>) -> impl Responder {
    match qclient
        .query(QueryPointsBuilder::new("qtact").query(vector.into_inner()))
        .await
    {
        Ok(response) => HttpResponse::Ok().body(format!("{:#?}", response)),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}
