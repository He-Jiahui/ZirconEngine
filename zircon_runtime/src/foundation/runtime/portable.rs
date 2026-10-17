//! Core owns configuration values on every target. Desktop persistence is a separate capability.
use crate::core::framework::foundation::{
    ConfigManager, ConfigManagerError, ConfigPersistenceReport,
};
use crate::core::{CoreError, CoreHandle, CoreWeak};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug)]
pub struct DefaultConfigManager {
    core: CoreWeak,
    generation: AtomicU64,
}
impl DefaultConfigManager {
    pub fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        Ok(Self {
            core: core.downgrade(),
            generation: AtomicU64::new(0),
        })
    }
}
impl ConfigManager for DefaultConfigManager {
    fn set_value(&self, key: &str, value: Value) -> Result<(), ConfigManagerError> {
        let core = self
            .core
            .upgrade()
            .ok_or(ConfigManagerError::RuntimeUnavailable)?;
        if core.store_config_value_if_changed(key, value) {
            self.generation.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }
    fn get_value(&self, key: &str) -> Option<Value> {
        self.core
            .upgrade()
            .and_then(|core| core.load_config_value(key))
    }
    fn flush(&self, _timeout: Duration) -> Result<(), ConfigManagerError> {
        Err(ConfigManagerError::Persistence { path: "runtime://configuration".into(),
            reason: "configuration is volatile on this target; no desktop file persistence backend is installed".into() })
    }
    fn persistence_report(&self) -> ConfigPersistenceReport {
        ConfigPersistenceReport {
            dirty_generation: self.generation.load(Ordering::Relaxed),
            last_error: Some("volatile runtime configuration".into()),
            ..Default::default()
        }
    }
}
