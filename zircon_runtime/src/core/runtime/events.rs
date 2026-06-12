//! Topic-based event distribution.

mod failure;
mod prune;
mod publish;
mod subscribe;

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::core::framework::channel::ChannelSender;
use crate::core::framework::events::EngineEvent;

#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<String, Arc<[ChannelSender<EngineEvent>]>>>>,
    delivery_lock: Arc<Mutex<()>>,
}

impl fmt::Debug for EventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventBus").finish()
    }
}
