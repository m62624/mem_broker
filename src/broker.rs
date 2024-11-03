use crate::topic::{Acknowledge, PublishMessage, Subscribe, Topic, Unsubscribe};
use actix::prelude::*;
use actix_web::Error;
use actix_web::{error, web, HttpResponse};
use futures::lock::Mutex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct Broker {
    topics: HashMap<String, Addr<Topic>>,
}

#[derive(Deserialize)]
pub struct CreateTopicRequest {
    pub name: String,
    pub retention: Option<u64>, // Время в секундах
    pub compaction: bool,
}

impl Broker {
    pub fn new() -> Broker {
        Broker {
            topics: HashMap::new(),
        }
    }

    pub fn create_topic(
        &mut self,
        name: String,
        retention: Option<Duration>,
        compaction: bool,
    ) -> Result<(), String> {
        if self.topics.contains_key(&name) {
            Err("Топик уже существует".into())
        } else {
            let topic = Topic::new(name.clone(), retention, compaction).start();
            self.topics.insert(name, topic);
            Ok(())
        }
    }

    pub fn publish_message(
        &self,
        topic_name: &str,
        message: crate::message::Message,
    ) -> Result<(), String> {
        if let Some(topic) = self.topics.get(topic_name) {
            topic.do_send(PublishMessage(message));
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }

    pub fn subscribe(
        &self,
        topic_name: &str,
        client_id: String,
        addr: Recipient<crate::topic::DeliverMessage>,
    ) -> Result<(), String> {
        if let Some(topic) = self.topics.get(topic_name) {
            topic.do_send(Subscribe { client_id, addr });
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }

    pub fn unsubscribe(&self, topic_name: &str, client_id: String) -> Result<(), String> {
        if let Some(topic) = self.topics.get(topic_name) {
            topic.do_send(Unsubscribe { client_id });
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }

    pub fn acknowledge(
        &self,
        topic_name: &str,
        client_id: String,
        message_id: String,
    ) -> Result<(), String> {
        if let Some(topic) = self.topics.get(topic_name) {
            topic.do_send(Acknowledge {
                client_id,
                message_id,
            });
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }
}

pub async fn create_topic_handler(
    broker: web::Data<Arc<Mutex<Broker>>>,
    req: web::Json<CreateTopicRequest>,
) -> Result<HttpResponse, Error> {
    let mut broker = broker.lock().await;
    broker
        .create_topic(
            req.name.clone(),
            req.retention.map(Duration::from_secs),
            req.compaction,
        )
        .map_err(error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().finish())
}
