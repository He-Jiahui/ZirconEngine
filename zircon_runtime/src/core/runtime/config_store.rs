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
        self.lock_values().insert(key.into(), Arc::new(value));
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

    /// 为诊断和外部观察者返回独立快照；后续写入不会改变已取得的结果。
    pub fn snapshot_values(&self) -> HashMap<String, Value> {
        let shared_entries = shared_snapshot_entries(&self.lock_values());
        shared_entries
            .into_iter()
            .map(|(key, value)| (key, value.as_ref().clone()))
            .collect()
    }
}

// 在配置锁内仅复制键与 Arc 所有权；调用方随后在锁外深拷贝 JSON，缩短大快照对写入的阻塞。
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
