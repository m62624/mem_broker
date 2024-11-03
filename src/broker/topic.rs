use super::{
    consumer::Consumer,
    message::{Confirm, Message, Subscribe, Unsubscribe},
};
use actix::{clock::Instant, Actor, Addr, AsyncContext, Context, Handler};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    time::Duration,
};

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
// Актор для топика
pub struct Topic {
    // Название топика
    name: String,
    // Время хранения сообщений
    retention: Option<Duration>,
    // Метод хранения сообщения
    messages: Compaction,
    // Те кто подписаны на этот топик
    consumers: HashMap<String, Addr<Consumer>>,
}

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
enum Compaction {
    // сохраняем только уникальные
    Enable(HashMap<String, MessageContainer>),
    // если id повторяются то сохраняем
    Disable(HashMap<String, VecDeque<MessageContainer>>),
}

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(PartialEq, Eq)]
struct MessageContainer {
    message: Message,
    reply: Option<HashSet<String>>, // Хранит клиентов, подтвердивших получение
}

impl Topic {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Topic {
            name: name.into(),
            retention: None,
            messages: Compaction::Disable(Default::default()),
            consumers: Default::default(),
        }
    }

    // Рассылка сообщения всем подписчикам
    pub fn broadcast(&mut self, mut message: Message) {
        // Добавляем временную метку, раз сообщение получено в топик
        message.timestamp_mut().replace(Instant::now());
        let send_message = message.clone();

        match &mut self.messages {
            Compaction::Enable(map) => {
                // Сохраняем всегда только последнее сообщение с таким id
                map.insert(
                    message.id().into(),
                    MessageContainer {
                        reply: if message.reply() {
                            Some(Default::default())
                        } else {
                            None
                        },
                        message,
                    },
                );
            }
            Compaction::Disable(map) => {
                map.entry(message.id().into())
                    .or_insert_with(Default::default)
                    .push_back(MessageContainer {
                        reply: if message.reply() {
                            Some(Default::default())
                        } else {
                            None
                        },
                        message,
                    });
            }
        }

        // Отправляем сообщение всем подписчикам
        self.consumers.values().for_each(|tx| {
            let _ = tx.try_send(send_message.clone());
        });
    }

    // Подтверждение получения сообщения
    pub fn confirm_message(&mut self, client_id: &str, message_id: &str) -> bool {
        let mut removed = false;
        match &mut self.messages {
            Compaction::Enable(map) => {
                if let Some(message) = map.get_mut(message_id) {
                    if let Some(reply) = message.reply.as_mut() {
                        // Если количество подтверждений равно количеству подписчиков, то удаляем сообщение
                        if reply.insert(client_id.into()) && reply.len() == self.consumers.len() {
                            map.remove(message_id);
                            removed = true;
                        }
                    }
                }
            }
            Compaction::Disable(map) => {
                if let Some(messages) = map.get_mut(message_id) {
                    for message in messages.iter_mut() {
                        if let Some(reply) = message.reply.as_mut() {
                            reply.insert(client_id.into());
                        }
                    }
                    messages.retain(|message| {
                        if let Some(reply) = &message.reply {
                            // Если количество подтверждений равно количеству подписчиков, то удаляем сообщение
                            reply.len() == self.consumers.len()
                        } else {
                            true
                        }
                    });
                    // Если коллекция сообщений пуста, то удаляем ключ
                    if messages.is_empty() {
                        map.remove(message_id);
                    }
                }
            }
        }
        removed
    }

    // Проверка времени хранения сообщений
    fn check_retantion(&mut self) {
        match &mut self.messages {
            Compaction::Enable(hash_map) => {
                let now = Instant::now();
                hash_map.retain(|_, message| {
                    if let Some(retention) = self.retention {
                        if let Some(t) = message.message.timestamp() {
                            now.duration_since(*t) < retention
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                });
            }
            Compaction::Disable(hash_map) => {
                let now = Instant::now();
                hash_map.retain(|_, messages| {
                    messages.retain(|message| {
                        if let Some(retention) = self.retention {
                            if let Some(t) = message.message.timestamp() {
                                now.duration_since(*t) < retention
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    });
                    !messages.is_empty()
                });
            }
        }
    }

    // Подписываем клиента на топик
    pub fn subscribe(&mut self, client_id: String, tx: Addr<Consumer>) {
        self.consumers.insert(client_id, tx);
    }

    // Отписываем клиента от топика
    pub fn unsubscribe(&mut self, client_id: &str) {
        self.consumers.remove(client_id);
    }
}

impl Actor for Topic {
    type Context = Context<Self>;
}

impl Handler<Message> for Topic {
    type Result = ();

    fn handle(&mut self, message: Message, ctx: &mut Context<Self>) -> Self::Result {
        self.broadcast(message);
        // Запускаем Interval для проверки времени хранения сообщений
        ctx.run_interval(
            self.retention.unwrap_or(Duration::from_secs(60)),
            |act, _ctx| {
                act.check_retantion();
            },
        );
    }
}

impl Handler<Confirm> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Confirm, _ctx: &mut Context<Self>) -> Self::Result {
        self.confirm_message(&msg.client_id, &msg.message_id);
    }
}

impl Handler<Subscribe> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Context<Self>) -> Self::Result {
        self.subscribe(msg.topic, msg.sender);
    }
}

impl Handler<Unsubscribe> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut Context<Self>) -> Self::Result {
        self.unsubscribe(&msg.0);
    }
}

impl Hash for MessageContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.message.id().hash(state);
    }
}
