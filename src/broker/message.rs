use super::*;
use actix::{clock::Instant, Message as ActixMessage};
use consumer::Consumer;
use serde::{Deserialize, Serialize};
pub use serde_json::Value;

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Message {
    id: String,
    reply: bool,
    payload: Value,
    #[serde(skip)]
    timestamp: Option<Instant>,
    #[serde(skip)]
    topic: Option<Addr<Topic>>,
}

pub struct Confirm {
    pub client_id: String,
    pub message_id: String,
}

pub struct Subscribe {
    pub topic: String,
    pub sender: Addr<Consumer>,
}

pub struct DeleteTopic;
pub struct Unsubscribe(pub String);

impl Message {
    pub fn new<T: Into<Value>>(payload: T) -> Self {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            reply: false,
            payload: payload.into(),
            timestamp: None,
            topic: None,
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

    pub fn topic(&self) -> &Option<Addr<Topic>> {
        &self.topic
    }
}

impl Subscribe {
    pub fn new<T: Into<String>>(topic: T, sender: Addr<Consumer>) -> Self {
        Subscribe {
            topic: topic.into(),
            sender,
        }
    }
}

impl Confirm {
    pub fn new<T: Into<String>, U: Into<String>>(client_id: T, message_id: U) -> Self {
        Confirm {
            client_id: client_id.into(),
            message_id: message_id.into(),
        }
    }
}

impl ActixMessage for Message {
    type Result = ();
}

impl ActixMessage for Confirm {
    type Result = ();
}

impl ActixMessage for Subscribe {
    type Result = ();
}

impl ActixMessage for Unsubscribe {
    type Result = ();
}
