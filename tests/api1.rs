use std::time::Duration;

use actix_web::{get, post, web, App, HttpServer, Responder, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    user_id: String,
    friend: String,
}

#[post("/create_topic")]
async fn create_topic() -> impl Responder {
    println!("create_topic");
    "create_topic"
}

/// extract path info using serde
#[get("/users/{user_id}/{friend}")] // <- define path parameters
async fn index(info: web::Path<Info>) -> Result<String> {
    Ok(format!(
        "Welcome {}, user_id {}!",
        info.friend, info.user_id
    ))
}

#[actix_web::test]
async fn test_t() -> std::io::Result<()> {
    let _ = HttpServer::new(|| App::new().service(index).service(create_topic))
        .bind("127.0.0.1:8080")?
        .run()
        .await;
    Ok(())
}
