//! Topic-based event distribution.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{ChannelReceiver, ChannelSender};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineEvent {
    pub topic: String,
    pub payload: Value,
}

#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<String, Vec<ChannelSender<EngineEvent>>>>>,
}

impl fmt::Debug for EventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventBus").finish()
    }
}

impl EventBus {
    pub fn subscribe(&self, topic: impl Into<String>) -> ChannelReceiver<EngineEvent> {
        let topic = topic.into();
        let (tx, rx) = unbounded();
        self.subscribers
            .lock()
            .unwrap()
            .entry(topic)
            .or_default()
            .push(tx);
        rx
    }

    pub fn publish(&self, event: EngineEvent) {
        if let Some(subscribers) = self.subscribers.lock().unwrap().get_mut(&event.topic) {
            subscribers.retain(|subscriber| subscriber.send(event.clone()).is_ok());
        }
    }
}
