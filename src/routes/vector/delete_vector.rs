use actix_web::{HttpResponse, Responder, delete, web};
use qdrant_client::{
    Qdrant,
    qdrant::{DeletePointsBuilder, PointsIdsList},
};

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
