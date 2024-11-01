use actix::{Actor, Addr};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashSet, hash::Hash};

use super::message;

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(Eq, PartialEq)]
pub(super) struct Topic<N, I, B>
where
    N: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    I: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    B: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
{
    name: N,
    subscribes: HashSet<message::Message<I, B>>,
}

impl<N, I, B> Topic<N, I, B>
where
    N: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    I: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    B: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
{
    pub fn new(name: N) -> Self {
        Self {
            name: name,
            subscribes: HashSet::new(),
        }
    }

    pub fn subscribe(&mut self, msg: message::Message<I, B>) {
        self.subscribes.insert(msg);
    }

    pub fn unsubscribe(&mut self, msg: message::Message<I, B>) {
        self.subscribes.remove(&msg);
    }
}

impl<N, I, B> Hash for Topic<N, I, B>
where
    N: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    I: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    B: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<N, I, B> Actor for Topic<N, I, B>
where
    N: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    I: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    B: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
{
    type Context = actix::Context<Self>;
}
