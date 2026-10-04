mod commit_fence;
mod state;
mod worker;
mod writer;

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use crate::core::framework::foundation::{
    ConfigManager, ConfigManagerError, ConfigPersistenceReport,
};
use crate::core::{CoreError, CoreHandle, CoreWeak};

use super::config_path::config_file_path;
use commit_fence::ConfigCommitFence;
use worker::ConfigPersistenceWorker;
use writer::{AtomicConfigFileWriter, ConfigFileWriter};

const DEFAULT_PERSISTENCE_DEBOUNCE: Duration = Duration::from_millis(25);
const DEFAULT_SHUTDOWN_FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct DefaultConfigManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    core: CoreWeak,
    path: Arc<PathBuf>,
    persistence: Arc<ConfigPersistenceWorker>,
}

impl DefaultConfigManager {
    pub fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        Self::new_with_options(
            core,
            config_file_path(),
            Arc::new(AtomicConfigFileWriter),
            DEFAULT_PERSISTENCE_DEBOUNCE,
            DEFAULT_SHUTDOWN_FLUSH_TIMEOUT,
        )
    }

    /// Constructs the normal persistence manager with immutable host-owned file authority.
    /// Explicit paths must be absolute; the default constructor retains its existing selector.
    pub fn new_with_file_path(core: &CoreHandle, path: PathBuf) -> Result<Self, CoreError> {
        Self::validate_file_path(&path)?;
        Self::new_with_options(
            core,
            path,
            Arc::new(AtomicConfigFileWriter),
            DEFAULT_PERSISTENCE_DEBOUNCE,
            DEFAULT_SHUTDOWN_FLUSH_TIMEOUT,
        )
    }

    pub(in crate::foundation) fn validate_file_path(path: &PathBuf) -> Result<(), CoreError> {
        if path.as_os_str().is_empty() || !path.is_absolute() {
            return Err(config_error(
                path,
                "explicit config persistence file path must be nonempty and absolute".to_owned(),
            ));
        }
        Ok(())
    }

    pub(super) fn new_with_options(
        core: &CoreHandle,
        path: PathBuf,
        writer: Arc<dyn ConfigFileWriter>,
        debounce: Duration,
        shutdown_timeout: Duration,
    ) -> Result<Self, CoreError> {
        let commit_fence = ConfigCommitFence::register(&path)
            .map_err(|error| config_error(&path, error.to_string()))?;
        recover_and_load_from_disk(core, &path, &commit_fence)?;
        let path = Arc::new(path);
        let persistence = ConfigPersistenceWorker::start(
            Arc::clone(&path),
            core.config_snapshot_source(),
            writer,
            debounce,
            shutdown_timeout,
            commit_fence,
        )?;
        Ok(Self {
            core: core.downgrade(),
            path,
            persistence,
        })
    }

    fn runtime_core(&self) -> Result<CoreHandle, ConfigManagerError> {
        self.core
            .upgrade()
            .ok_or(ConfigManagerError::RuntimeUnavailable)
    }
}

impl fmt::Debug for DefaultConfigManager {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DefaultConfigManager")
            .field("path", &self.path)
            .field("persistence", &self.persistence)
            .finish()
    }
}

impl ConfigManager for DefaultConfigManager {
    fn set_value(&self, key: &str, value: Value) -> Result<(), ConfigManagerError> {
        let core = self.runtime_core()?;
        let changed = store_config_value_if_changed(&core, key, value);
        self.persistence.request_persistence(changed);
        Ok(())
    }

    fn get_value(&self, key: &str) -> Option<Value> {
        self.core
            .upgrade()
            .and_then(|core| core.load_config_value(key))
    }

    fn flush(&self, timeout: Duration) -> Result<(), ConfigManagerError> {
        self.persistence.flush(timeout)
    }

    fn persistence_report(&self) -> ConfigPersistenceReport {
        self.persistence.report()
    }
}

pub(super) fn store_config_value_if_changed(core: &CoreHandle, key: &str, value: Value) -> bool {
    if core.load_config_value(key).as_ref() == Some(&value) {
        return false;
    }
    core.store_config_value(key.to_string(), value);
    true
}

fn recover_and_load_from_disk(
    core: &CoreHandle,
    path: &PathBuf,
    commit_fence: &ConfigCommitFence,
) -> Result<(), CoreError> {
    let json = commit_fence
        .commit(|| {
            crate::core::resource::io::recover_missing_target_from_backup(path)?;
            match fs::read_to_string(path) {
                Ok(json) => Ok(Some(json)),
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error),
            }
        })
        .map_err(|error| config_error(path, error.to_string()))?;
    let Some(json) = json else {
        return Ok(());
    };
    let values = serde_json::from_str::<HashMap<String, Value>>(&json)
        .map_err(|error| config_error(path, error.to_string()))?;
    for (key, value) in values {
        core.store_config_value(key, value);
    }
    Ok(())
}

fn config_error(path: &PathBuf, message: String) -> CoreError {
    CoreError::ConfigParse(path.to_string_lossy().into_owned(), message)
}

#[cfg(test)]
pub(super) use commit_fence::ConfigCommitFence as ConfigCommitFenceForTest;
#[cfg(test)]
pub(super) use writer::ConfigFileWriter as ConfigFileWriterForTest;
