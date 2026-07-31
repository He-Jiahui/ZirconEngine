use serde_json::Value;

use crate::core::framework::events::{EngineEventDeliveryPolicy, EngineEventSubscription};

pub trait EventManager: Send + Sync {
    fn publish(&self, topic: &str, payload: Value);
    fn subscribe(
        &self,
        topic: &str,
        policy: EngineEventDeliveryPolicy,
    ) -> Box<dyn EngineEventSubscription>;
}
