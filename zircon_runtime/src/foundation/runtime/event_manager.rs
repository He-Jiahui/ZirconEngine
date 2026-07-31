use std::sync::Arc;
use std::time::Duration;

use crate::core::framework::events::{
    EngineEvent, EngineEventDeliveryPolicy, EngineEventReceiveError,
    EngineEventReceiveTimeoutError, EngineEventSubscription, EngineEventTryReceiveError,
};
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

    fn subscribe(
        &self,
        topic: &str,
        policy: EngineEventDeliveryPolicy,
    ) -> Box<dyn EngineEventSubscription> {
        if let Some(core) = self.core.upgrade() {
            return core.subscribe_events(topic.to_string(), policy);
        }
        Box::new(DisconnectedEventSubscription)
    }
}

struct DisconnectedEventSubscription;

impl EngineEventSubscription for DisconnectedEventSubscription {
    fn recv(&self) -> Result<Arc<EngineEvent>, EngineEventReceiveError> {
        Err(EngineEventReceiveError::Disconnected)
    }

    fn try_recv(&self) -> Result<Arc<EngineEvent>, EngineEventTryReceiveError> {
        Err(EngineEventTryReceiveError::Disconnected)
    }

    fn recv_timeout(
        &self,
        _timeout: Duration,
    ) -> Result<Arc<EngineEvent>, EngineEventReceiveTimeoutError> {
        Err(EngineEventReceiveTimeoutError::Disconnected)
    }
}
