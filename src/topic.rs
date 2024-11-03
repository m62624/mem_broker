use crate::message::Message;
use actix::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

// Структура топика
pub struct Topic {
    // Название топика
    // name: String,
    // Время хранения сообщений
    retention: Option<Duration>,
    // Флаг компакции, если true, то мы используем last_message_by_key
    // чтобы хранить последнее сообщение для каждого ключа
    compaction: bool,
    // Сообщения, используем VecDeque для быстрого доступа к началу и концу,
    // так как мы будем удалять старые сообщения, а также добавлять новые в конец
    messages: VecDeque<Message>,
    // Последнее сообщение для каждого ключа
    last_message_by_key: HashMap<String, Message>,
    // Подписчики на топик
    subscribers: HashMap<String, Recipient<DeliverMessage>>,
    // Ожидающие подтверждения сообщения
    pending_acks: HashMap<String, HashSet<String>>, // message_id -> set of client_ids
}

// Сообщение для публикации
#[derive(Message)]
#[rtype(result = "()")]
pub struct PublishMessage(pub Message);

// Сообщение для подписки на топик
#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub client_id: String,
    pub addr: Recipient<DeliverMessage>,
}

// Сообщение для отписки от топика
#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribe {
    pub client_id: String,
}

// Сообщение для подтверждения получения сообщения
#[derive(Message)]
#[rtype(result = "()")]
pub struct Acknowledge {
    pub client_id: String,
    pub message_id: String,
}

// Сообщение для доставки сообщения
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeliverMessage(pub Message);

impl Topic {
    pub fn new(retention: Option<Duration>, compaction: bool) -> Self {
        Topic {
            // name,
            retention,
            compaction,
            messages: VecDeque::new(),
            last_message_by_key: HashMap::new(),
            subscribers: HashMap::new(),
            pending_acks: HashMap::new(),
        }
    }

    // Очистка старых сообщений
    fn clean_up_messages(&mut self) {
        // Если установлено время хранения сообщений, запускаем цикл
        // и удаляем старые сообщения
        if let Some(retention_duration) = self.retention {
            // Получаем текущее время
            let now = Instant::now();

            // Начинаем с начала очереди сообщений, самые старые сообщения
            while let Some(message) = self.messages.front() {
                // Если сообщение не имеет времени, то мы не можем его удалить
                if let Some(timestamp) = message.timestamp {
                    if now.duration_since(timestamp) > retention_duration {
                        // Удаляем сообщение
                        self.messages.pop_front();
                    }
                } else {
                    break;
                }
            }
        }
    }

    // Отправка сообщения подписчикам, + проверка на подтверждение (если требуется)
    fn deliver_message(&mut self, message: &Message, ctx: &mut Context<Self>) {
        // рассылаем сообщение подписчикам
        for (client_id, subscriber) in &self.subscribers {
            let _ = subscriber.do_send(DeliverMessage(message.clone()));

            // Если сообщение требует подтверждения, добавляем в ожидающие
            if message.require_ack {
                self.pending_acks
                    .entry(message.id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(client_id.clone());
            }
        }

        if message.require_ack {
            let message_id = message.id.clone();
            // тут делаем spawn, чтобы не блокировать текущий контекст
            // через n секунд проверяем, что все получили сообщение
            ctx.run_later(Duration::from_secs(30), move |act, ctx| {
                act.check_pending_ack(message_id.clone(), ctx);
            });
        }
    }

    // Проверка на подтверждение получения сообщения
    fn check_pending_ack(&mut self, message_id: String, _: &mut Context<Self>) {
        if let Some(client_ids) = self.pending_acks.get(&message_id) {
            if !client_ids.is_empty() {
                // Если не все получили сообщение, повторяем отправку
                for client_id in client_ids {
                    if let Some(subscriber) = self.subscribers.get(client_id) {
                        let _ = subscriber.do_send(DeliverMessage(
                            self.last_message_by_key[&message_id].clone(),
                        ));
                    }
                }
            } else {
                self.pending_acks.remove(&message_id);
            }
        }
    }
}

impl Actor for Topic {
    type Context = Context<Self>;

    // Запускаем таймер для очистки старых сообщений, каждую минуту
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(60), |act, _| {
            act.clean_up_messages();
        });
    }
}

impl Handler<PublishMessage> for Topic {
    type Result = ();

    // Обработка сообщения для публикации
    fn handle(&mut self, msg: PublishMessage, ctx: &mut Self::Context) -> Self::Result {
        let message = msg.0;

        // Если включена компакция, то мы храним последнее сообщение для каждого ключа
        if self.compaction {
            if let Some(key) = &message.key {
                self.last_message_by_key
                    .insert(key.clone(), message.clone());
            }
        } else {
            // Иначе просто добавляем сообщение в очередь
            self.messages.push_back(message.clone());
        }

        // Отправляем сообщение подписчикам
        self.deliver_message(&message, ctx);
    }
}

// Обработка сообщения для подписки
impl Handler<Subscribe> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        // Добавляем подписчика
        self.subscribers.insert(msg.client_id, msg.addr);
    }
}

// Обработка сообщения для отписки
impl Handler<Unsubscribe> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut Self::Context) -> Self::Result {
        // Удаляем подписчика
        self.subscribers.remove(&msg.client_id);
    }
}

// Обработка сообщения для подтверждения получения сообщения
impl Handler<Acknowledge> for Topic {
    type Result = ();

    fn handle(&mut self, msg: Acknowledge, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(client_ids) = self.pending_acks.get_mut(&msg.message_id) {
            // Удаляем клиента из ожидающих
            client_ids.remove(&msg.client_id);
            // Если все получили сообщение, удаляем из ожидающих
            if client_ids.is_empty() {
                self.pending_acks.remove(&msg.message_id);
            }
        }
    }
}
