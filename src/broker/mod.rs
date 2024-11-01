mod message;
mod topic;
mod user;

use actix::{Actor, Addr};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
struct Broker<N, I, B>
where
    N: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    I: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
    B: Hash + Serialize + DeserializeOwned + Unpin + Eq + 'static,
{
    topics: Arc<tokio::sync::Mutex<HashMap<N, Addr<topic::Topic<N, I, B>>>>>,
}
