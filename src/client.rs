#[cfg(not(feature = "remote"))]
use crate::broker::Broker;
use crate::topic::Topic;
#[cfg(not(feature = "remote"))]
use actix::Recipient;

#[cfg(not(feature = "remote"))]
use actix::Addr;

use actix::{Actor, Context, Handler};

#[cfg(feature = "remote")]
use futures::channel::mpsc;

use super::message::Message;
use crate::broker::Id;

use std::collections::VecDeque;

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
pub struct Client {
    id: Id,
    messages: VecDeque<Message>,

    // Broker
    #[cfg(not(feature = "remote"))]
    connection: Addr<Broker>,

    #[cfg(feature = "remote")]
    connection: (
        mpsc::UnboundedSender<Message>,
        Option<mpsc::UnboundedReceiver<Message>>,
    ),
}

impl Client {
    pub fn new(#[cfg(not(feature = "remote"))] broker: Addr<Broker>) -> Self {
        Self {
            id: Id(uuid::Uuid::new_v4().into()),
            messages: VecDeque::new(),
            #[cfg(feature = "remote")]
            connection: {
                let (tx, rx) = mpsc::unbounded();
                (tx, Some(rx))
            },
            #[cfg(not(feature = "remote"))]
            connection: broker,
        }
    }
}

impl Actor for Client {
    type Context = Context<Self>;
}

impl Handler<Message> for Client {
    type Result = ();

    fn handle(&mut self, msg: Message, _ctx: &mut Self::Context) {
        self.messages.push_back(msg);
    }
}

pub mod message {
    use actix::ResponseFuture;

    use crate::{broker::Id, topic::Topic};

    pub struct NewTopic(pub Topic);

    pub struct CreateTopic(pub Id, pub Topic);

    // client id
    pub struct DeleteTopic(pub Id, pub String);

    pub struct GetTopics;

    // client id
    pub struct GetListOfTopics(pub Id);

    impl actix::Message for NewTopic {
        type Result = ();
    }

    #[cfg(not(feature = "remote"))]
    impl actix::Handler<NewTopic> for super::Client {
        type Result = ();

        fn handle(&mut self, msg: NewTopic, _ctx: &mut Self::Context) -> Self::Result {
            self.connection.do_send(CreateTopic(self.id.clone(), msg.0));
        }
    }

    impl actix::Message for CreateTopic {
        type Result = ();
    }

    #[cfg(not(feature = "remote"))]
    impl actix::Handler<CreateTopic> for super::Client {
        type Result = ();

        fn handle(&mut self, msg: CreateTopic, _ctx: &mut Self::Context) -> Self::Result {
            self.connection.do_send(msg);
        }
    }

    impl actix::Message for DeleteTopic {
        type Result = ();
    }

    #[cfg(not(feature = "remote"))]
    impl actix::Handler<DeleteTopic> for super::Client {
        type Result = ();

        fn handle(&mut self, msg: DeleteTopic, _ctx: &mut Self::Context) -> Self::Result {
            self.connection.do_send(msg);
        }
    }

    impl actix::Message for GetTopics {
        type Result = Vec<String>;
    }

    #[cfg(not(feature = "remote"))]
    impl actix::Handler<GetTopics> for super::Client {
        type Result = ResponseFuture<Vec<String>>;

        fn handle(&mut self, _msg: GetTopics, _ctx: &mut Self::Context) -> Self::Result {
            let connection = self.connection.clone();
            let id = self.id.clone();
            Box::pin(async move {
                connection
                    .send(GetListOfTopics(id))
                    .await
                    .unwrap_or_else(|_| Vec::new())
            })
        }
    }

    impl actix::Message for GetListOfTopics {
        type Result = Vec<String>;
    }

    #[cfg(not(feature = "remote"))]
    impl actix::Handler<GetListOfTopics> for super::Client {
        type Result = ResponseFuture<Vec<String>>;

        fn handle(&mut self, _msg: GetListOfTopics, _ctx: &mut Self::Context) -> Self::Result {
            let connection = self.connection.clone();
            let id = self.id.clone();
            Box::pin(async move {
                connection
                    .send(GetListOfTopics(id))
                    .await
                    .unwrap_or_else(|_| Vec::new())
            })
        }
    }
}
