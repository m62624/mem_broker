use actix::{Actor, Context, Handler};

use super::message::{Confirm, Message};

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
pub struct Consumer {
    client_id: String,
    vector: Vec<Message>,
}

impl Consumer {
    pub fn new<T: Into<String>>(client_id: T) -> Self {
        Consumer {
            client_id: client_id.into(),
            vector: Vec::new(),
        }
    }
}

impl Actor for Consumer {
    type Context = Context<Self>;
}

impl Handler<Message> for Consumer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Self::Context) -> Self::Result {
        if msg.reply() {
            if let Some(topic) = msg.topic() {
                topic.do_send(Confirm::new(&self.client_id, msg.id()));
            }
        }
        self.vector.push(msg);
    }
}
