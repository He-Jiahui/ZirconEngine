//! JSON config storage.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::core::CoreError;

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
    fn lock_values(&self) -> MutexGuard<'_, HashMap<String, Value>> {
        self.values
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn store_value(&self, key: impl Into<String>, value: Value) {
        self.lock_values().insert(key.into(), value);
    }

    pub fn load_value(&self, key: &str) -> Option<Value> {
        self.lock_values().get(key).cloned()
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
        self.lock_values().clone()
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    fn poison_values_lock(store: &ConfigStore) {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = store.values.lock().unwrap();
            panic!("poison config store values lock");
        }));
        assert!(result.is_err());
    }

    #[test]
    fn config_store_accessors_recover_poisoned_values_lock() {
        let store = ConfigStore::default();

        store.store_value("before", Value::from(1));
        poison_values_lock(&store);

        store.store_value("after", Value::from(2));
        assert_eq!(store.load_value("before"), Some(Value::from(1)));
        assert_eq!(store.load::<u64>("after").unwrap(), 2);

        let snapshot = store.snapshot_values();
        assert_eq!(snapshot.get("before"), Some(&Value::from(1)));
        assert_eq!(snapshot.get("after"), Some(&Value::from(2)));
    }
}
