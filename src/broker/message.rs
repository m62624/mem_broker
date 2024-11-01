use actix::{self, Actor};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use std::hash::Hash;
use std::time::Instant;

/// Структура сообщения с обобщённым типом `T`.

#[cfg_attr(any(feature = "debug", test), derive(Debug))]
#[derive(Serialize, Hash, Eq, PartialEq)]
pub struct Message<I, B>
where
    I: Hash + Serialize + DeserializeOwned + Eq,
    B: Hash + Serialize + DeserializeOwned + Eq,
{
    id: Option<I>,
    body: B,
    /// Счетчик подтверждений (может использоваться для отслеживания доставки).
    commit: Option<u64>,

    /// Время хранения сообщения. Пропускается при сериализации и устанавливается в None при десериализации.
    #[serde(skip)]
    retention: Option<Instant>,
}

impl<I, B> Message<I, B>
where
    I: Hash + Serialize + DeserializeOwned + Eq,
    B: Hash + Serialize + DeserializeOwned + Eq,
{
    pub fn new(body: B) -> Self {
        Self {
            id: None,
            body,
            commit: None,
            retention: None,
        }
    }

    pub fn set_id(&mut self, id: I) {
        self.id = Some(id);
    }

    pub(super) fn update_retention_time(&mut self) {
        self.retention = Some(Instant::now());
    }

    pub(super) fn has_expired(&self, duration: std::time::Duration) -> bool {
        if let Some(time) = self.retention {
            time.elapsed() > duration
        } else {
            false
        }
    }

    pub(super) fn take_id(&mut self) -> Option<I> {
        self.id.take()
    }
}

// Ручная реализация Deserialize из за borrow checker.
impl<'de, I, B> Deserialize<'de> for Message<I, B>
where
    I: Hash + Serialize + DeserializeOwned + Unpin + Eq,
    B: Hash + Serialize + DeserializeOwned + Unpin + Eq,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MessageHelper<I, B> {
            id: Option<I>,
            body: B,
            commit: Option<u64>,
        }

        let helper = MessageHelper::deserialize(deserializer)?;

        Ok(Self {
            id: helper.id,
            body: helper.body,
            commit: helper.commit,
            retention: None,
        })
    }
}
