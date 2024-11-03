use futures::channel::mpsc;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::time::Instant;

use super::message::Message;

// Актор для топика
pub struct Topic {
    // Название топика
    name: String,
    // Время хранения сообщений
    retention: Option<Duration>,
    // Метод хранения сообщения
    messages: Compaction,
    // Те кто подписаны на этот топик
    consumers: HashMap<String, mpsc::UnboundedSender<Message>>,
}

enum Compaction {
    // сохраняем только уникальные
    Enable(HashMap<String, MessageContainer>),
    // если id повторяются то просто стакаем
    Disable(HashMap<String, VecDeque<MessageContainer>>),
}

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

    pub fn broadcast(&mut self, mut message: Message) {
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
            let _ = tx.unbounded_send(send_message.clone());
        });
    }

    pub fn confirm_message(&mut self, client_id: &str, message_id: &str) -> bool {
        let mut r = false;
        match &mut self.messages {
            Compaction::Enable(map) => {
                if let Some(message) = map.get_mut(message_id) {
                    if let Some(reply) = message.reply.as_mut() {
                        reply.insert(client_id.into());
                        if reply.len() == self.consumers.len() {
                            map.remove(message_id);
                            r = true;
                        }
                    }
                }
            }
            Compaction::Disable(map) => {
                if let Some(messages) = map.get_mut(message_id) {
                    messages.iter_mut().for_each(|message| {
                        if let Some(reply) = message.reply.as_mut() {
                            reply.insert(client_id.into());
                        }
                    });
                    messages.retain(|message| {
                        if let Some(reply) = message.reply.as_ref() {
                            if reply.len() == self.consumers.len() {
                                r = true;
                                false
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    });
                }
            }
        }
        r
    }

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
    pub fn subscribe(&mut self, client_id: String, tx: mpsc::UnboundedSender<Message>) {
        self.consumers.insert(client_id, tx);
    }

    // Отписываем клиента от топика
    pub fn unsubscribe(&mut self, client_id: &str) {
        self.consumers.remove(client_id);
    }
}

impl Hash for MessageContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.message.id().hash(state);
    }
}
