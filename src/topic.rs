use crate::message::Message;
use actix::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

#[derive(Message)]
#[rtype(result = "()")]
pub struct PublishMessage(pub Message);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub client_id: String,
    pub addr: Recipient<DeliverMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribe {
    pub client_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Acknowledge {
    pub client_id: String,
    pub message_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DeliverMessage(pub Message);

pub struct Topic {
    name: String,
    retention: Option<Duration>,
    compaction: bool,
    messages: VecDeque<Message>,
    last_message_by_key: HashMap<String, Message>,
    subscribers: HashMap<String, Recipient<DeliverMessage>>,
    pending_acks: HashMap<String, HashSet<String>>, // message_id -> set of client_ids
}

impl Topic {
    pub fn new(name: String, retention: Option<Duration>, compaction: bool) -> Self {
        Topic {
            name,
            retention,
            compaction,
            messages: VecDeque::new(),
            last_message_by_key: HashMap::new(),
            subscribers: HashMap::new(),
            pending_acks: HashMap::new(),
        }
    }

    fn clean_up_messages(&mut self) {
        if let Some(retention_duration) = self.retention {
            let now = Instant::now();
            while let Some(message) = self.messages.front() {
                if let Some(timestamp) = message.timestamp {
                    if now.duration_since(timestamp) > retention_duration {
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    fn deliver_message(&mut self, message: &Message, ctx: &mut Context<Self>) {
        for (client_id, subscriber) in &self.subscribers {
            let _ = subscriber.do_send(DeliverMessage(message.clone()));

            if message.require_ack {
                self.pending_acks
                    .entry(message.id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(client_id.clone());
            }
        }

        if message.require_ack {
            let message_id = message.id.clone();
            ctx.run_later(Duration::from_secs(30), move |act, _| {
                act.check_pending_ack(message_id);
            });
        }
    }

    fn check_pending_ack(&mut self, message_id: String) {
        if let Some(client_ids) = self.pending_acks.get(&message_id) {
            if !client_ids.is_empty() {
                // Повторная отправка или логирование недоставленных сообщений
            } else {
                self.pending_acks.remove(&message_id);
            }
        }
    }
}

impl Actor for Topic {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(60), |act, _| {
            act.clean_up_messages();
        });
    }
}

impl Handler<PublishMessage> for Topic {
    type Result = ();

    fn handle(&mut self, msg: PublishMessage, ctx: &mut Self::Context) -> Self::Result {
        let message = msg.0;

        if self.compaction {
            if let Some(key) = &message.key {
                self.last_message_by_key
                    .insert(key.clone(), message.clone());
            }
        } else {
            self.messages.push_back(message.clone());
        }

        self.deliver_message(&message, ctx);
    }
}

impl Handler<Subscribe> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        self.subscribers.insert(msg.client_id, msg.addr);
    }
}

impl Handler<Unsubscribe> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut Self::Context) -> Self::Result {
        self.subscribers.remove(&msg.client_id);
    }
}

impl Handler<Acknowledge> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Acknowledge, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(client_ids) = self.pending_acks.get_mut(&msg.message_id) {
            client_ids.remove(&msg.client_id);
            if client_ids.is_empty() {
                self.pending_acks.remove(&msg.message_id);
            }
        }
    }
}
