//! JSON config storage.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::error::CoreError;

#[derive(Clone, Default)]
pub struct ConfigStore {
    values: Arc<Mutex<HashMap<String, Value>>>,
}

impl fmt::Debug for ConfigStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigStore").finish()
    }
}

impl ConfigStore {
    pub fn store_value(&self, key: impl Into<String>, value: Value) {
        self.values.lock().unwrap().insert(key.into(), value);
    }

    pub fn load_value(&self, key: &str) -> Option<Value> {
        self.values.lock().unwrap().get(key).cloned()
    }

    pub fn store<T: Serialize>(&self, key: impl Into<String>, value: &T) -> Result<(), CoreError> {
        let key = key.into();
        let value = serde_json::to_value(value)
            .map_err(|error| CoreError::ConfigParse(key.clone(), error.to_string()))?;
        self.store_value(key, value);
        Ok(())
    }

    pub fn load<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        let value = self
            .load_value(key)
            .ok_or_else(|| CoreError::MissingConfig(key.to_string()))?;
        serde_json::from_value(value)
            .map_err(|error| CoreError::ConfigParse(key.to_string(), error.to_string()))
    }

    pub fn snapshot_values(&self) -> HashMap<String, Value> {
        self.values.lock().unwrap().clone()
    }
}
