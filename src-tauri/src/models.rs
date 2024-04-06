use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentData {
    pub id: String,
    pub text: String,
    pub sentiment: String,
    pub score: f64,
}