use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, post, web};
use data_model::vector::Vector;
use env_logger::Env;
use qdrant::create_qdrant_client;
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, Distance, GetPointsBuilder, PointStruct, UpsertPointsBuilder,
        VectorParamsBuilder,
    },
};

mod data_model;
mod qdrant;

// API Routes
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/create")]
async fn create_vector(qclient: web::Data<Qdrant>, vector: web::Json<Vector>) -> impl Responder {
    let vector = vector.into_inner();
    let point = PointStruct {
        id: Some(vector.id.into()),
        vectors: Some(vector.vector.clone().into()),
        payload: vector.payload,
    };
    match qclient
        .upsert_points(UpsertPointsBuilder::new("qtact", vec![point]).wait(true))
        .await
    {
        Ok(_) => HttpResponse::Ok().body("Vector created"),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error = {}", err)),
    }
}

#[get("/get/{id}")]
async fn get_vector(qclient: web::Data<Qdrant>, vector_id: web::Path<u64>) -> impl Responder {
    match qclient
        .get_points(
            GetPointsBuilder::new("qtact", vec![vector_id.into_inner().into()])
                .with_vectors(true)
                .with_payload(true),
        )
        .await
    {
        Ok(point) => HttpResponse::Ok().json(serde_json::from_reader(point)),
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
    let res = qclient
        .create_collection(
            CreateCollectionBuilder::new("qtact")
                .vectors_config(VectorParamsBuilder::new(512, Distance::Cosine)),
        )
        .await
        .unwrap();

    // Server
    HttpServer::new(|| App::new().wrap(Logger::default()).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
