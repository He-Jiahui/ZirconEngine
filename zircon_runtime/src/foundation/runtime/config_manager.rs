use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::framework::foundation::ConfigManager;
use crate::core::{CoreError, CoreHandle, CoreWeak};

use super::config_path::config_file_path;

#[derive(Clone, Debug)]
pub struct DefaultConfigManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    core: CoreWeak,
    path: Arc<PathBuf>,
}

impl DefaultConfigManager {
    pub fn new(core: &CoreHandle) -> Self {
        let manager = Self {
            core: core.downgrade(),
            path: Arc::new(config_file_path()),
        };
        manager.load_from_disk(core);
        manager
    }

    fn runtime_core(&self) -> Result<CoreHandle, CoreError> {
        self.core.upgrade().ok_or(CoreError::RuntimeUnavailable)
    }

    fn load_from_disk(&self, core: &CoreHandle) {
        let Ok(json) = fs::read_to_string(self.path.as_path()) else {
            return;
        };
        let Ok(values) = serde_json::from_str::<HashMap<String, Value>>(&json) else {
            return;
        };
        for (key, value) in values {
            core.store_config_value(key, value);
        }
    }

    fn persist_to_disk(&self, core: &CoreHandle) -> Result<(), CoreError> {
        let values = core.snapshot_config_values();
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|error| {
                    CoreError::ConfigParse(
                        self.path.to_string_lossy().into_owned(),
                        error.to_string(),
                    )
                })?;
            }
        }
        fs::write(
            self.path.as_path(),
            serde_json::to_string_pretty(&values).map_err(|error| {
                CoreError::ConfigParse(self.path.to_string_lossy().into_owned(), error.to_string())
            })?,
        )
        .map_err(|error| {
            CoreError::ConfigParse(self.path.to_string_lossy().into_owned(), error.to_string())
        })
    }
}

impl ConfigManager for DefaultConfigManager {
    fn set_value(&self, key: &str, value: Value) -> Result<(), CoreError> {
        let core = self.runtime_core()?;
        core.store_config_value(key.to_string(), value);
        self.persist_to_disk(&core)
    }

    fn get_value(&self, key: &str) -> Option<Value> {
        self.core
            .upgrade()
            .and_then(|core| core.load_config_value(key))
    }
}
