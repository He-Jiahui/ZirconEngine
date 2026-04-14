use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use zircon_core::{
    ChannelReceiver, CoreError, CoreHandle, DriverDescriptor, EngineEvent, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_module::{factory, qualified_name};

use crate::{ConfigManager, ConfigManagerHandle, EventManager, EventManagerHandle};

pub const MANAGER_MODULE_NAME: &str = "ManagerModule";
pub const CONFIG_DRIVER_NAME: &str = "ManagerModule.Driver.ConfigDriver";
pub const CONFIG_MANAGER_NAME: &str = "ManagerModule.Manager.ConfigManager";
pub const EVENT_DRIVER_NAME: &str = "ManagerModule.Driver.EventDriver";
pub const EVENT_MANAGER_NAME: &str = "ManagerModule.Manager.EventManager";

#[derive(Debug, Default)]
pub struct ConfigDriver;

#[derive(Debug, Default)]
pub struct EventDriver;

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

#[derive(Clone, Debug)]
pub struct DefaultEventManager {
    core: CoreHandle,
}

impl DefaultEventManager {
    pub fn new(core: CoreHandle) -> Self {
        Self { core }
    }
}

impl EventManager for DefaultEventManager {
    fn publish(&self, topic: &str, payload: Value) {
        self.core.publish_event(topic.to_string(), payload);
    }

    fn subscribe(&self, topic: &str) -> ChannelReceiver<EngineEvent> {
        self.core.subscribe_events(topic.to_string())
    }
}

fn config_file_path() -> PathBuf {
    if let Some(path) = std::env::var_os("ZIRCON_EDITOR_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if cfg!(target_os = "windows") {
        if let Some(base) = std::env::var_os("LOCALAPPDATA").or_else(|| std::env::var_os("APPDATA"))
        {
            return PathBuf::from(base)
                .join("ZirconEngine")
                .join("editor-config.json");
        }
    } else if let Some(base) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(base)
            .join("ZirconEngine")
            .join("editor-config.json");
    } else if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("ZirconEngine")
            .join("editor-config.json");
    }

    PathBuf::from(".zircon-editor-config.json")
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MANAGER_MODULE_NAME, "Stable manager facade layer")
        .with_driver(DriverDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Driver, "ConfigDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(ConfigDriver) as ServiceObject)),
        ))
        .with_driver(DriverDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Driver, "EventDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(EventDriver) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Manager, "ConfigManager"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|core| {
                let manager = Arc::new(DefaultConfigManager::new(core.clone()));
                Ok(Arc::new(ConfigManagerHandle::new(manager)) as ServiceObject)
            }),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(MANAGER_MODULE_NAME, ServiceKind::Manager, "EventManager"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|core| {
                let manager = Arc::new(DefaultEventManager::new(core.clone()));
                Ok(Arc::new(EventManagerHandle::new(manager)) as ServiceObject)
            }),
        ))
}
