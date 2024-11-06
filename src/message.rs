use crate::*;
use actix::clock::Instant;
use actix::Message as ActixMessage;
use broker::Id;
use serde_json::Value;


#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct Message {
    pub id: Id,
    pub key: Option<String>,
    pub payload: Value,
    #[serde(skip)]
    pub timestamp: Option<Instant>,
    require_ack: bool,
}

impl Message {
    pub fn new(payload: Value) -> Self {
        Self {
            id: Id(uuid::Uuid::new_v4().into()),
            key: None,
            payload,
            timestamp: None,
            require_ack: false,
        }
    }

    pub fn require_ack(&mut self) {
        self.require_ack = true;
    }

    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    pub fn with_timestamp(mut self, timestamp: Instant) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn change_id(&mut self, id: String) {
        self.id = Id(id);
    }
}

impl ActixMessage for Message {
    type Result = ();
}

mod message_actor {
    use super::Message;

    pub struct BroadcastMessage(Message);

    impl BroadcastMessage {
        pub fn new(message: Message) -> Self {
            Self(message)
        }
    }

    pub struct SubscribeOnTopic(String);

    impl SubscribeOnTopic {
        pub fn new<T: Into<String>>(topic: T) -> Self {
            Self(topic.into())
        }
    }

    pub struct UnsubscribeFromTopic(String);

    impl UnsubscribeFromTopic {
        pub fn new<T: Into<String>>(topic: T) -> Self {
            Self(topic.into())
        }
    }

    pub struct AcknowledgeMessage(String);

    impl AcknowledgeMessage {
        pub fn new<T: Into<String>>(message_id: T) -> Self {
            Self(message_id.into())
        }
    }
}
