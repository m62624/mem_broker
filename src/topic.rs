use crate::broker::Id;
use crate::message::Message;
use actix::Message as ActixMessage;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

#[cfg(not(feature = "remote"))]
use actix::ActorContext;
#[cfg(not(feature = "remote"))]
use actix::Handler;

use actix::{Actor, Context, Recipient};

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
pub struct Topic {
    name: String,
    retention: Option<Duration>,
    // id-key
    compaction: Option<HashMap<Id, Message>>,
    // id-client
    subscribers: HashSet<Recipient<Message>>,
    messages: VecDeque<Message>,
    #[cfg(feature = "remote")]
    // id-message, id-client
    pending_ack: HashMap<Id, HashSet<Id>>,
}

impl Topic {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            retention: None,
            compaction: None,
            subscribers: HashSet::new(),
            messages: VecDeque::new(),
            #[cfg(feature = "remote")]
            pending_ack: HashMap::new(),
        }
    }

    pub fn with_retention(mut self, retention: Duration) -> Self {
        self.retention = Some(retention);
        self
    }

    pub fn with_compaction(mut self) -> Self {
        self.compaction = Some(HashMap::new());
        self
    }

    pub fn subscribe(&mut self, client: Recipient<Message>) {
        self.subscribers.insert(client);
    }

    pub fn unsubscribe(&mut self, client: Recipient<Message>) {
        self.subscribers.remove(&client);
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Actor for Topic {
    type Context = Context<Self>;
}

pub mod message {

    use super::*;

    pub struct Stop;

    impl ActixMessage for Stop {
        type Result = ();
    }

    #[cfg(not(feature = "remote"))]
    impl Handler<Stop> for Topic {
        type Result = ();

        fn handle(&mut self, _msg: Stop, ctx: &mut Self::Context) {
            ctx.stop();
        }
    }
}
