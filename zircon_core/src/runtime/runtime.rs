use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config_store::ConfigStore;
use crate::error::CoreError;
use crate::event_bus::{EngineEvent, EventBus};
use crate::job_scheduler::JobScheduler;
use crate::types::ChannelReceiver;

use super::handle::CoreHandle;
use super::state::CoreRuntimeInner;
use super::weak::CoreWeak;
use super::ModuleDescriptor;

#[derive(Clone)]
pub struct CoreRuntime {
    inner: Arc<CoreRuntimeInner>,
}

impl CoreRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CoreRuntimeInner {
                modules: Default::default(),
                services: Default::default(),
                event_bus: EventBus::default(),
                config_store: ConfigStore::default(),
                scheduler: JobScheduler::default(),
            }),
        }
    }

    pub fn handle(&self) -> CoreHandle {
        CoreHandle {
            inner: self.inner.clone(),
        }
    }

    pub fn weak(&self) -> CoreWeak {
        self.handle().downgrade()
    }

    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        self.handle().register_module(descriptor)
    }

    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle().activate_module(module_name)
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle().deactivate_module(module_name)
    }

    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle().resolve_driver(name)
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle().resolve_manager(name)
    }

    pub fn publish_event(&self, topic: impl Into<String>, payload: Value) {
        self.handle().publish_event(topic, payload)
    }

    pub fn subscribe_events(&self, topic: impl Into<String>) -> ChannelReceiver<EngineEvent> {
        self.handle().subscribe_events(topic)
    }

    pub fn store_config_value(&self, key: impl Into<String>, value: Value) {
        self.handle().store_config_value(key, value)
    }

    pub fn load_config_value(&self, key: &str) -> Option<Value> {
        self.handle().load_config_value(key)
    }

    pub fn snapshot_config_values(&self) -> HashMap<String, Value> {
        self.handle().snapshot_config_values()
    }

    pub fn load_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        self.handle().load_config(key)
    }
}

impl Default for CoreRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CoreRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreRuntime").finish()
    }
}
