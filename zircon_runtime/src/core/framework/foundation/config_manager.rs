use std::time::Duration;

use serde_json::Value;

use crate::core::CoreError;

use super::ConfigPersistenceReport;

pub trait ConfigManager: Send + Sync {
    fn set_value(&self, key: &str, value: Value) -> Result<(), CoreError>;
    fn get_value(&self, key: &str) -> Option<Value>;
    fn flush(&self, timeout: Duration) -> Result<(), CoreError>;
    fn persistence_report(&self) -> ConfigPersistenceReport;

    fn contains_key(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }
}
