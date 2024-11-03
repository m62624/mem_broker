use actix_web::{web, App, HttpServer};

use super::broker::{message::*, Consumer, MemBroker};

pub async fn http_server() {
    let mut broker = web::Data::new(MemBroker::new());

    HttpServer::new(move || App::new().app_data(broker.clone()));
}
