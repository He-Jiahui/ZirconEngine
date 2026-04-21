use crate::core::{ChannelReceiver, CoreError, EngineEvent};
use serde_json::Value;

pub trait ConfigManager: Send + Sync {
    fn set_value(&self, key: &str, value: Value) -> Result<(), CoreError>;
    fn get_value(&self, key: &str) -> Option<Value>;

    fn contains_key(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }
}

pub trait EventManager: Send + Sync {
    fn publish(&self, topic: &str, payload: Value);
    fn subscribe(&self, topic: &str) -> ChannelReceiver<EngineEvent>;
}
