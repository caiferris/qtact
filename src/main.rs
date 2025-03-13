use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::Env;
use qdrant::create_qdrant_client;
use qdrant_client::qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder};
use routes::{
    index::index,
    vector::{
        create_vector::create_vector, delete_vector::delete_vector, filter_vector::filter_vector,
        get_vector::get_vector, similar_vector::similar_vector, update_vector::update_vector,
    },
};

mod data_model;
mod qdrant;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Enable logging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Connect to Qdrant Vector DB
    let qclient = create_qdrant_client().await;

    //  Create collection
    match qclient.collection_exists("qtact").await {
        Ok(true) => {
            dbg!("Collection qtact exists, using the existing");
        }
        Ok(false) => {
            let res = qclient
                .create_collection(
                    CreateCollectionBuilder::new("qtact")
                        .vectors_config(VectorParamsBuilder::new(4, Distance::Cosine)),
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
                    .service(delete_vector)
                    .service(similar_vector)
                    .service(filter_vector),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
