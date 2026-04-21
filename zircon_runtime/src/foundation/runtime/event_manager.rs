use crate::core::framework::foundation::EventManager;
use crate::core::{ChannelReceiver, CoreHandle, EngineEvent};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct DefaultEventManager {
    core: CoreHandle,
}

impl DefaultEventManager {
    pub fn new(core: CoreHandle) -> Self {
        Self { core }
    }
}

impl EventManager for DefaultEventManager {
    fn publish(&self, topic: &str, payload: Value) {
        self.core.publish_event(topic.to_string(), payload);
    }

    fn subscribe(&self, topic: &str) -> ChannelReceiver<EngineEvent> {
        self.core.subscribe_events(topic.to_string())
    }
}
