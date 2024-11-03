use actix_web::{web, App, HttpServer};
use futures::lock::Mutex;
use mem_broker::{broker::Broker, client::init_routes};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let broker = Arc::new(Mutex::new(Broker::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(broker.clone())) // Общие данные
            .configure(init_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
