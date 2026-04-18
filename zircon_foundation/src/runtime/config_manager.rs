use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use zircon_core::{CoreError, CoreHandle};
use zircon_framework::foundation::ConfigManager;

use super::config_path::config_file_path;

#[derive(Clone, Debug)]
pub struct DefaultConfigManager {
    core: CoreHandle,
    path: Arc<PathBuf>,
}

impl DefaultConfigManager {
    pub fn new(core: CoreHandle) -> Self {
        let manager = Self {
            core,
            path: Arc::new(config_file_path()),
        };
        manager.load_from_disk();
        manager
    }

    fn load_from_disk(&self) {
        let Ok(json) = fs::read_to_string(self.path.as_path()) else {
            return;
        };
        let Ok(values) = serde_json::from_str::<HashMap<String, Value>>(&json) else {
            return;
        };
        for (key, value) in values {
            self.core.store_config_value(key, value);
        }
    }

    fn persist_to_disk(&self) -> Result<(), CoreError> {
        let values = self.core.snapshot_config_values();
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
        self.core.store_config_value(key.to_string(), value);
        self.persist_to_disk()
    }

    fn get_value(&self, key: &str) -> Option<Value> {
        self.core.load_config_value(key)
    }
}
