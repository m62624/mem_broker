use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub key: Option<String>,
    pub payload: String,
    pub require_ack: bool,
    #[serde(skip)]
    pub timestamp: Option<Instant>,
}

impl Message {
    pub fn new(payload: String, key: Option<String>, require_ack: bool) -> Self {
        Message {
            id: Uuid::new_v4().to_string(),
            key,
            payload,
            require_ack,
            timestamp: Some(Instant::now()),
        }
    }
}
