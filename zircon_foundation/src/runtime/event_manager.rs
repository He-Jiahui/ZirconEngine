use serde_json::Value;
use zircon_core::{ChannelReceiver, CoreHandle, EngineEvent};
use zircon_framework::foundation::EventManager;

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
