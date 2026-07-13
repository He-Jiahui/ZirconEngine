use crate::core::framework::channel::ChannelReceiver;
use crate::core::framework::events::EngineEvent;
use crate::core::framework::foundation::EventManager;
use crate::core::{CoreHandle, CoreWeak};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct DefaultEventManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    core: CoreWeak,
}

impl DefaultEventManager {
    pub fn new(core: &CoreHandle) -> Self {
        Self {
            core: core.downgrade(),
        }
    }
}

impl EventManager for DefaultEventManager {
    fn publish(&self, topic: &str, payload: Value) {
        if let Some(core) = self.core.upgrade() {
            core.publish_event(topic.to_string(), payload);
        }
    }

    fn subscribe(&self, topic: &str) -> ChannelReceiver<EngineEvent> {
        if let Some(core) = self.core.upgrade() {
            return core.subscribe_events(topic.to_string());
        }
        let (_sender, receiver) = crossbeam_channel::unbounded();
        receiver
    }
}
