use crate::{
    broker::{Broker, CreateTopicRequest},
    message::Message,
    topic::DeliverMessage,
};
use actix::prelude::*;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use futures::{channel::mpsc, lock::Mutex, StreamExt};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

// Структура для хранения сессии клиента, хранит отправителя сообщений
struct ClientSession {
    tx: mpsc::UnboundedSender<Message>,
}

// Структура для публикации сообщения
#[derive(Deserialize)]
pub struct PublishRequest {
    topic: String,
    key: Option<String>,
    payload: String,
    require_ack: bool,
}

// Структура для подписки на топик
#[derive(Deserialize)]
pub struct SubscribeRequest {
    topic: String,
}

#[derive(Deserialize)]
pub struct UnsubscribeRequest {
    pub topic: String,
}

// Структура для подтверждения получения сообщения
#[derive(Deserialize)]
pub struct AcknowledgeRequest {
    topic: String,
    client_id: String,
    message_id: String,
}

// Функция для публикации сообщения
pub async fn publish(
    broker: web::Data<Arc<Mutex<Broker>>>,
    req: web::Json<PublishRequest>,
) -> Result<HttpResponse, Error> {
    let broker = broker.lock().await;
    let message = Message::new(req.payload.clone(), req.key.clone(), req.require_ack);
    broker
        .publish_message(&req.topic, message)
        .map_err(error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().finish())
}

// Функция для подтверждения получения сообщения
pub async fn acknowledge(
    broker: web::Data<Arc<Mutex<Broker>>>,
    req: web::Json<AcknowledgeRequest>,
) -> Result<HttpResponse, Error> {
    broker
        .lock()
        .await
        .acknowledge(&req.topic, req.client_id.clone(), req.message_id.clone())
        .map_err(error::ErrorBadRequest)?;

    Ok(HttpResponse::Ok().finish())
}

// Функция для подписки на топик
pub async fn subscribe(
    broker: web::Data<Arc<Mutex<Broker>>>,
    // _req: HttpRequest,
    // _stream: web::Payload,
    path: web::Query<SubscribeRequest>,
) -> Result<HttpResponse, Error> {
    // Создаем уникальный идентификатор клиента
    let client_id = Uuid::new_v4();

    let (tx, rx) = mpsc::unbounded();

    let ws = ClientSession { tx };
    let addr = ws.start();

    {
        let broker = broker.lock().await;
        broker
            .subscribe(&path.topic, client_id.to_string(), addr.recipient())
            .map_err(error::ErrorBadRequest)?;

        println!("Клиент подписался на топик {}", path.topic);
        println!("ID клиента: {}", client_id);
    }

    let res = HttpResponse::Ok()
        .insert_header(("content-type", "text/event-stream"))
        .streaming(rx.map(|msg| Ok::<_, Error>(web::Bytes::from(msg.payload))));

    Ok(res)
}

pub async fn unsubscribe(
    broker: web::Data<Arc<Mutex<Broker>>>,
    req: web::Json<UnsubscribeRequest>,
    req_http: HttpRequest,
) -> Result<HttpResponse, Error> {
    // Извлекаем client_id из заголовков
    let client_id = req_http
        .headers()
        .get("X-Client-Id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| error::ErrorBadRequest("Отсутствует заголовок X-Client-Id"))?
        .to_string();

    // Обрабатываем отписку
    {
        let broker = broker.lock().await;
        broker
            .unsubscribe(&req.topic, client_id)
            .map_err(error::ErrorBadRequest)?;
    }

    Ok(HttpResponse::Ok().finish())
}

// Функция для создания топика
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

// Изменяем настройки маршрутов
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/publish").route(web::post().to(publish)))
        .service(web::resource("/subscribe").route(web::get().to(subscribe)))
        .service(web::resource("/unsubscribe").route(web::post().to(unsubscribe)))
        .service(web::resource("/ack").route(web::post().to(acknowledge)))
        .service(web::resource("/create_topic").route(web::post().to(create_topic_handler)));
}

impl Actor for ClientSession {
    type Context = Context<Self>;
}

impl Handler<DeliverMessage> for ClientSession {
    type Result = ();

    fn handle(&mut self, msg: DeliverMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let _ = self.tx.unbounded_send(msg.0);
    }
}
