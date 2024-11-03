use actix_web::http::header;
use actix_web::web::{Data, Path};
use actix_web::{web, App, HttpResponse, HttpServer};
use futures::channel::mpsc;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Тип для хранения соединений SSE
type Clients = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>;

// SSE обработчик для подключения клиента
async fn sse_connect(clients: Data<Clients>, client_id: Path<String>) -> HttpResponse {
    let (tx, rx) = mpsc::unbounded::<String>();
    let id = client_id.into_inner();

    // Сохраняем `Sender` для данного клиента
    clients.lock().unwrap().insert(id.clone(), tx);

    // Устанавливаем заголовки для SSE
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/event-stream"))
        .append_header((header::CACHE_CONTROL, "no-cache"))
        .streaming(rx.map(|msg| Ok::<_, actix_web::Error>(web::Bytes::from(msg))))
}

// Отправка события конкретному клиенту по его ID
async fn send_event(clients: Data<Clients>, client_id: Path<String>, msg: String) -> HttpResponse {
    let clients = clients.lock().unwrap();
    if let Some(sender) = clients.get(&client_id.into_inner()) {
        // Отправляем сообщение, если клиент существует
        let _ = sender.unbounded_send(msg);
        HttpResponse::Ok().body("Message sent")
    } else {
        HttpResponse::NotFound().body("Client not found")
    }
}

// Отправка сообщения всем клиентам
async fn broadcast(clients: Data<Clients>, msg: String) -> HttpResponse {
    for sender in clients.lock().unwrap().values() {
        let _ = sender.unbounded_send(msg.clone());
    }
    HttpResponse::Ok().body("Broadcast message sent")
}

#[actix_web::test]
async fn test_1() -> std::io::Result<()> {
    let clients = Data::new(Clients::new(Mutex::new(HashMap::new())));

    HttpServer::new(move || {
        App::new()
            .app_data(clients.clone())
            .route("/sse/{client_id}", web::get().to(sse_connect))
            .route("/send/{client_id}", web::post().to(send_event))
            .route("/broadcast", web::post().to(broadcast))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
