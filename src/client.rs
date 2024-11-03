use std::sync::Arc;
use std::time::Duration;

use crate::broker::Broker;
use crate::message::Message;
use crate::topic::DeliverMessage;
use actix::prelude::*;
use actix_web::{error, web, Error, HttpResponse};
use futures::channel::mpsc;
use futures::lock::Mutex;
use futures::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateTopicRequest {
    pub name: String,
    pub retention: Option<u64>, // Будут секундами
    pub compaction: bool,
}

#[derive(Deserialize)]
pub struct PublishRequest {
    topic: String,
    key: Option<String>,
    payload: String,
    require_ack: bool,
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    topic: String,
}

#[derive(Deserialize)]
pub struct AcknowledgeRequest {
    topic: String,
    client_id: String,
    message_id: String,
}

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

pub async fn subscribe(
    broker: web::Data<Arc<Mutex<Broker>>>,
    // _req: HttpRequest,
    // _stream: web::Payload,
    path: web::Query<SubscribeRequest>,
) -> Result<HttpResponse, Error> {
    let client_id = Uuid::new_v4();
    let (tx, rx) = mpsc::unbounded();

    let ws = ClientSession { tx };
    let addr = ws.start();

    {
        let broker = broker.lock().await;
        broker
            .subscribe(&path.topic, client_id.to_string(), addr.recipient())
            .map_err(error::ErrorBadRequest)?;
    }

    let res = HttpResponse::Ok()
        .insert_header(("content-type", "text/event-stream"))
        .streaming(rx.map(|msg| Ok::<_, Error>(web::Bytes::from(msg.payload))));

    Ok(res)
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

struct ClientSession {
    tx: mpsc::UnboundedSender<Message>,
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

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/publish").route(web::post().to(publish)))
        .service(web::resource("/subscribe").route(web::get().to(subscribe)))
        .service(web::resource("/ack").route(web::post().to(acknowledge)))
        .service(web::resource("/create_topic").route(web::post().to(create_topic_handler)));
}
