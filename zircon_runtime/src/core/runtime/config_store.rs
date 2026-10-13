//! JSON config storage.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::CoreError;

/// CoreRuntime 共享的配置快照入口；克隆句柄共享同一存储，读取返回独立值。
///
/// 启动入口先写入配置，驱动和编辑器再经 CoreHandle 读取；解析失败由调用方处理。
#[derive(Clone, Default)]
pub struct ConfigStore {
    values: Arc<Mutex<HashMap<String, Arc<Value>>>>,
}

impl fmt::Debug for ConfigStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigStore").finish()
    }
}

impl ConfigStore {
    fn lock_values(&self) -> MutexGuard<'_, HashMap<String, Arc<Value>>> {
        self.values
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn store_value(&self, key: impl Into<String>, value: Value) {
        let key = key.into();
        let value = Arc::new(value);
        let retired = {
            let mut values = self.lock_values();
            values.insert(key, value)
        };
        drop(retired);
    }

    // Deep comparison does not block readers. Retain the observed owner until publication so
    // a concurrent replacement cannot reuse its identity and invalidate the changed decision.
    pub(crate) fn store_value_if_changed(&self, key: &str, value: Value) -> bool {
        self.store_value_if_changed_by(key, value, Value::eq)
    }

    fn store_value_if_changed_by(
        &self,
        key: &str,
        value: Value,
        equals: impl Fn(&Value, &Value) -> bool,
    ) -> bool {
        let mut observed = self.shared_value(key);
        loop {
            if !observed
                .as_ref()
                .is_some_and(|stored| equals(stored.as_ref(), &value))
            {
                break;
            }
            let current = self.shared_value(key);
            if same_shared_owner(current.as_ref(), observed.as_ref()) {
                return false;
            }
            observed = current;
        }

        let owned_key = key.to_owned();
        let value = Arc::new(value);
        let mut unchanged = false;
        loop {
            let mut values = self.lock_values();
            let current = values.get(key);
            if same_shared_owner(current, observed.as_ref()) {
                if unchanged {
                    drop(values);
                    return false;
                }
                let replaced = values.insert(owned_key, value);
                drop(values);
                drop(replaced);
                return true;
            }

            let current = current.cloned();
            drop(values);
            // Retire a stale observation and repeat deep equality after releasing the mutex.
            observed = current;
            unchanged = observed
                .as_ref()
                .is_some_and(|stored| equals(stored.as_ref(), value.as_ref()));
        }
    }

    pub fn load_value(&self, key: &str) -> Option<Value> {
        self.shared_value(key).map(|value| value.as_ref().clone())
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
            .shared_value(key)
            .ok_or_else(|| CoreError::MissingConfig(key.to_string()))?;
        T::deserialize(value.as_ref())
            .map_err(|error| CoreError::ConfigParse(key.to_string(), error.to_string()))
    }

    // 只在锁内取得共享所有权；反序列化和深拷贝不占用全局配置锁。
    fn shared_value(&self, key: &str) -> Option<Arc<Value>> {
        self.lock_values().get(key).cloned()
    }

    // Persistence can serialize retained immutable owners without copying their JSON payloads.
    pub(crate) fn shared_snapshot_values(&self) -> HashMap<String, Arc<Value>> {
        let shared_entries = shared_snapshot_entries(&self.lock_values());
        shared_entries.into_iter().collect()
    }

    /// 为诊断和外部观察者返回独立快照；后续写入不会改变已取得的结果。
    pub fn snapshot_values(&self) -> HashMap<String, Value> {
        let shared_entries = shared_snapshot_entries(&self.lock_values());
        shared_entries
            .into_iter()
            .map(|(key, value)| (key, value.as_ref().clone()))
            .collect()
    }
}

fn same_shared_owner(current: Option<&Arc<Value>>, observed: Option<&Arc<Value>>) -> bool {
    match (current, observed) {
        (Some(current), Some(observed)) => Arc::ptr_eq(current, observed),
        (None, None) => true,
        _ => false,
    }
}

// 在配置锁内仅复制键与 Arc 所有权；调用方随后在锁外构造或处理快照。
fn shared_snapshot_entries(values: &HashMap<String, Arc<Value>>) -> Vec<(String, Arc<Value>)> {
    values
        .iter()
        .map(|(key, value)| (key.clone(), Arc::clone(value)))
        .collect()
}

#[cfg(test)]
#[path = "config_store/tests/shared_snapshot_tests.rs"]
mod shared_snapshot_tests;

#[cfg(test)]
#[path = "config_store/tests/shared_load_tests.rs"]
mod shared_load_tests;

#[cfg(test)]
#[path = "tests/config_store.rs"]
mod tests;
