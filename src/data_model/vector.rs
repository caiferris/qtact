use std::collections::HashMap;

use qdrant_client::qdrant::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Vector {
    pub id: u64,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, Value>,
}
