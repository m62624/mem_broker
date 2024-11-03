use crate::topic::{Acknowledge, PublishMessage, Subscribe, Topic, Unsubscribe};
use actix::prelude::*;
use actix_web::Error;
use actix_web::{error, web, HttpResponse};
use futures::lock::Mutex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Хранение топиков (каждый топик будет актором)
pub struct Broker {
    topics: HashMap<String, Addr<Topic>>,
}

// Структура для создания топика
#[derive(Deserialize)]
pub struct CreateTopicRequest {
    pub name: String,
    // Время после которого сообщения удаляются
    pub retention: Option<u64>, // Время в секундах
    pub compaction: bool,
}

impl Broker {
    pub fn new() -> Broker {
        Broker {
            topics: HashMap::new(),
        }
    }

    // Создание нового топика
    pub fn create_topic(
        &mut self,
        name: String,
        retention: Option<Duration>,
        compaction: bool,
    ) -> Result<(), String> {
        if self.topics.contains_key(&name) {
            Err("Топик уже существует".into())
        } else {
            println!("Топик создан - {}", name);
            // Создаем новый топик и переводим в актор
            self.topics.insert(
                name.clone(),
                Topic::new(name, retention, compaction).start(),
            );
            Ok(())
        }
    }

    // Отправка сообщения в топик
    pub fn publish_message(
        &self,
        topic_name: &str,
        message: crate::message::Message,
    ) -> Result<(), String> {
        if let Some(topic) = self.topics.get(topic_name) {
            println!("ID сообщения: {}", message.id);
            topic.do_send(PublishMessage(message));
            println!("Сообщение отправлено");
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }

    pub fn subscribe(
        &self,
        topic_name: &str,
        client_id: String,
        // Recipient - это адресат сообщения
        addr: Recipient<crate::topic::DeliverMessage>,
    ) -> Result<(), String> {
        // Если топик существует, отправляем сообщение, что клиент подписался
        if let Some(topic) = self.topics.get(topic_name) {
            topic.do_send(Subscribe { client_id, addr });
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }

    // Отписка от топика
    pub fn unsubscribe(&self, topic_name: &str, client_id: String) -> Result<(), String> {
        // Если топик существует, отправляем сообщение, что клиент отписался
        if let Some(topic) = self.topics.get(topic_name) {
            topic.do_send(Unsubscribe { client_id });
            println!("Клиент отписался");
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }

    // Подтверждение получения сообщения
    pub fn acknowledge(
        &self,
        topic_name: &str,
        client_id: String,
        message_id: String,
    ) -> Result<(), String> {
        // Если топик существует, отправляем сообщение, что сообщение получено
        if let Some(topic) = self.topics.get(topic_name) {
            topic.do_send(Acknowledge {
                client_id,
                message_id,
            });
            println!("Сообщение получено");
            Ok(())
        } else {
            Err("Топик не найден".into())
        }
    }
}

// Обработчик создания топика
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
