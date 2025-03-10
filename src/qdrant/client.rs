use qdrant_client::Qdrant;

pub async fn create_qdrant_client() -> Qdrant {
    Qdrant::from_url("http://localhost:6333").build().unwrap()
}
