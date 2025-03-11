use actix_web::{
    App, HttpResponse, HttpServer, Responder, delete, get, middleware::Logger, post, put, web,
};
use data_model::vector::Vector;
use env_logger::Env;
use qdrant::create_qdrant_client;
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, DeletePointsBuilder, Distance, GetPointsBuilder, PointStruct,
        PointsIdsList, UpsertPointsBuilder, VectorParamsBuilder,
    },
};

mod data_model;
mod qdrant;

// API Routes
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().finish()
}

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
            dbg!("Response = {:?}", &response);
            HttpResponse::Ok().body(format!("Vector created = {:?}", response))
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
            dbg!("Point = {:?}", &response);
            HttpResponse::Ok().body(format!("{:?}", response))
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
            dbg!("Point Updated = {:?}", &response);
            HttpResponse::Ok().body(format!("{:?}", response))
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
            HttpResponse::Ok().body(format!("Deleted vector = {:?}", response))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Enable logging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Connect to Qdrant Vector DB
    let qclient = create_qdrant_client().await;

    //  Create collection
    match qclient.collection_exists("qtact").await {
        Ok(true) => (),
        Ok(false) => {
            let res = qclient
                .create_collection(
                    CreateCollectionBuilder::new("qtact")
                        .vectors_config(VectorParamsBuilder::new(3, Distance::Cosine)),
                )
                .await
                .unwrap();
            dbg!("Create Collection Response = {:?}", res);
        }
        Err(err) => {
            panic!("{}", err);
        }
    }

    let qclient = web::Data::new(qclient);

    // Server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&qclient))
            .wrap(Logger::default())
            .service(
                // Scoped for v1 APIs
                web::scope("/v1")
                    .service(index)
                    .service(create_vector)
                    .service(get_vector)
                    .service(update_vector)
                    .service(delete_vector),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
