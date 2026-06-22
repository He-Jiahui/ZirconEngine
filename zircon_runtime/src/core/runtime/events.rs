//! Topic-based event distribution.

mod failure;
mod prune;
mod publish;
mod subscribe;

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::channel::ChannelSender;
use crate::core::framework::events::EngineEvent;

type EventSubscriberMap = HashMap<String, Arc<[ChannelSender<EngineEvent>]>>;

#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<EventSubscriberMap>>,
    delivery_lock: Arc<Mutex<()>>,
}

impl EventBus {
    fn lock_subscribers(&self) -> MutexGuard<'_, EventSubscriberMap> {
        self.subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_delivery(&self) -> MutexGuard<'_, ()> {
        self.delivery_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl fmt::Debug for EventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventBus").finish()
    }
}
