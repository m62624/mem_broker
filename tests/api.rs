use actix::prelude::*;

// Определяем тип сообщения
struct MyMessage {
    content: String,
}

// Реализуем трейт Message для MyMessage
impl Message for MyMessage {
    type Result = ();
}

// Определяем клиентскую структуру
struct Client {
    id: String,
    recipient: Recipient<MyMessage>, // Здесь ожидаем Recipient с типом сообщения MyMessage
}

impl Client {
    pub fn new(id: String, recipient: Recipient<MyMessage>) -> Self {
        Self { id, recipient }
    }

    // Метод для отправки сообщения
    pub fn send_message(&self, content: String) {
        let message = MyMessage { content };
        // Отправляем сообщение на Recipient
        self.recipient.do_send(message);
    }
}

// Пример актора, который может обрабатывать сообщения
struct MyActor;

impl Actor for MyActor {
    type Context = Context<Self>;
}

// Реализуем обработчик для MyMessage
impl Handler<MyMessage> for MyActor {
    type Result = ();

    fn handle(&mut self, msg: MyMessage, _: &mut Self::Context) {
        println!("Received message: {}", msg.content);
    }
}

#[actix::test]
async fn test_t_0() {
    let my_actor = MyActor.start();
    let recipient: Recipient<MyMessage> = my_actor.recipient();

    let client = Client::new("Client1".to_string(), recipient);
    client.send_message("Hello, Actor!".to_string());
}
