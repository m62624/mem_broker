use super::*;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Message {
    id: String,
    reply: bool,
    payload: Value,
    #[serde(skip)]
    timestamp: Option<Instant>,
}

impl Message {
    pub fn new<T: Into<Value>>(payload: T) -> Self {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            reply: false,
            payload: payload.into(),
            timestamp: None,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn with_reply(mut self) -> Self {
        self.reply = true;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn reply(&self) -> bool {
        self.reply
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }

    pub fn timestamp(&self) -> &Option<Instant> {
        &self.timestamp
    }

    pub fn timestamp_mut(&mut self) -> &mut Option<Instant> {
        &mut self.timestamp
    }
}
