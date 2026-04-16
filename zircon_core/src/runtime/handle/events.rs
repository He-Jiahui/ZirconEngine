use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::CoreError;
use crate::event_bus::EngineEvent;
use crate::types::ChannelReceiver;

use super::CoreHandle;

impl CoreHandle {
    pub fn publish_event(&self, topic: impl Into<String>, payload: Value) {
        self.inner.event_bus.publish(EngineEvent {
            topic: topic.into(),
            payload,
        });
    }

    pub fn subscribe_events(&self, topic: impl Into<String>) -> ChannelReceiver<EngineEvent> {
        self.inner.event_bus.subscribe(topic)
    }

    pub fn store_config_value(&self, key: impl Into<String>, value: Value) {
        self.inner.config_store.store_value(key, value);
    }

    pub fn load_config_value(&self, key: &str) -> Option<Value> {
        self.inner.config_store.load_value(key)
    }

    pub fn snapshot_config_values(&self) -> HashMap<String, Value> {
        self.inner.config_store.snapshot_values()
    }

    pub fn store_config<T: serde::Serialize>(
        &self,
        key: impl Into<String>,
        value: &T,
    ) -> Result<(), CoreError> {
        self.inner.config_store.store(key, value)
    }

    pub fn load_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        self.inner.config_store.load(key)
    }
}
