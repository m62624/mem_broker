use super::client;

use actix::{Addr, Handler};
use std::collections::HashMap;

#[cfg(not(feature = "remote"))]
use actix::Actor;
#[cfg(not(feature = "remote"))]
use actix::Context;
#[cfg(not(feature = "remote"))]
use actix::ResponseFuture;

use super::*;
use crate::topic::Topic;

pub struct Broker {
    // id-client
    topics: HashMap<Id, HashMap<String, Addr<Topic>>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[cfg_attr(any(feature = "debug", test), derive(Debug))]
pub struct Id(pub String);

impl Broker {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
        }
    }
}

#[cfg(not(feature = "remote"))]
impl Actor for Broker {
    type Context = Context<Self>;
}

#[cfg(not(feature = "remote"))]
impl Handler<client::message::CreateTopic> for Broker {
    type Result = ();

    fn handle(&mut self, msg: client::message::CreateTopic, _ctx: &mut Self::Context) {
        self.topics
            .entry(msg.0)
            .or_insert_with(HashMap::new)
            .insert(msg.1.name().into(), msg.1.start());
    }
}

#[cfg(not(feature = "remote"))]
impl Handler<client::message::DeleteTopic> for Broker {
    type Result = ();

    fn handle(&mut self, msg: client::message::DeleteTopic, _ctx: &mut Self::Context) {
        if let Some(t_n) = self.topics.get_mut(&msg.0) {
            if let Some(topic) = t_n.remove(&msg.1) {
                topic.do_send(topic::message::Stop);
            }
        }
    }
}

#[cfg(not(feature = "remote"))]
impl Handler<client::message::GetListOfTopics> for Broker {
    type Result = Vec<String>;

    fn handle(
        &mut self,
        msg: client::message::GetListOfTopics,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let mut topics = Vec::new();

        if let Some(t_n) = self.topics.get(&msg.0) {
            let t_n = t_n.clone();
            for (name, _) in t_n {
                topics.push(name.clone());
            }
        }
        topics
    }
}
