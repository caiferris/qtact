// Narrow down the results by filtering by the payload

use actix_web::{HttpResponse, Responder, get, web};
use qdrant_client::{
    Qdrant,
    qdrant::{Condition, Filter, QueryPointsBuilder},
};

#[get("/vectorfilter")]
async fn filter_vector(qclient: web::Data<Qdrant>, vector: web::Json<Vec<f32>>) -> impl Responder {
    match qclient
        .query(
            QueryPointsBuilder::new("qtact")
                .query(vector.into_inner())
                .filter(Filter::must([Condition::matches(
                    "city",
                    "London".to_string(),
                )])),
        )
        .await
    {
        Ok(response) => {
            dbg!("Query Response = {:#?}", &response);
            HttpResponse::Ok().body(format!("{:#?}", response))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}
