mod consumer;
pub mod message;
mod topic;

pub use actix::Actor;
use actix::Addr;
use futures::lock::Mutex;
use std::collections::HashMap;
use std::hash::Hash;
use topic::Topic;

pub use consumer::Consumer;

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
pub struct MemBroker {
    topics: Vec<Addr<Topic>>,
    id_table: HashMap<String, usize>,
}

impl MemBroker {
    pub fn new() -> Self {
        MemBroker {
            topics: Default::default(),
            id_table: Default::default(),
        }
    }

    pub async fn create_topic<T: Into<String>>(&mut self, name: T) -> Addr<Topic> {
        let name: String = name.into();
        let topic = Topic::new(name.clone()).start();
        // self.topics.push(Mutex::new(topic.clone()));
        self.id_table.insert(name, self.topics.len() - 1);
        topic
    }

    pub async fn delete_topic(&mut self, name: String) {
        if let Some(index) = self.id_table.get(&name) {
            self.topics.remove(*index);
            self.id_table.remove(&name);
        }
    }

    // pub fn topics(&self) -> &Vec<Mutex<Addr<Topic>>> {
    //     // &self.topics
    // }
}
