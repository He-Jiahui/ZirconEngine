use serde_json::Value;
use zircon_core::{ChannelReceiver, CoreError, EngineEvent};
use zircon_input_protocol::{InputEvent, InputEventRecord, InputSnapshot};
use zircon_resource::{ResourceEvent, ResourceRecord};
use zircon_scene_protocol::{LevelSummary, WorldHandle};

use crate::RenderingBackendInfo;

pub trait RenderingManager: Send + Sync {
    fn backend_info(&self) -> RenderingBackendInfo;
}

pub trait LevelManager: Send + Sync {
    fn create_default_level_handle(&self) -> WorldHandle;
    fn level_exists(&self, handle: WorldHandle) -> bool;
    fn level_summary(&self, handle: WorldHandle) -> Option<LevelSummary>;
    fn load_level_asset(&self, project_root: &str, uri: &str) -> Result<WorldHandle, CoreError>;
    fn save_level_asset(
        &self,
        handle: WorldHandle,
        project_root: &str,
        uri: &str,
    ) -> Result<(), CoreError>;
}

pub trait ResourceManager: Send + Sync {
    fn resolve_resource_id(&self, locator: &str) -> Option<String>;
    fn resource_status(&self, locator: &str) -> Option<ResourceRecord>;
    fn list_resources(&self) -> Vec<ResourceRecord>;
    fn resource_revision(&self, locator: &str) -> Option<u64>;
    fn subscribe_resource_changes(&self) -> ChannelReceiver<ResourceEvent>;
}

pub trait InputManager: Send + Sync {
    fn submit_event(&self, event: InputEvent);
    fn snapshot(&self) -> InputSnapshot;
    fn drain_events(&self) -> Vec<InputEvent>;
    fn drain_event_records(&self) -> Vec<InputEventRecord>;
}

pub trait ConfigManager: Send + Sync {
    fn set_value(&self, key: &str, value: Value) -> Result<(), CoreError>;
    fn get_value(&self, key: &str) -> Option<Value>;

    fn contains_key(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }
}

pub trait EventManager: Send + Sync {
    fn publish(&self, topic: &str, payload: Value);
    fn subscribe(&self, topic: &str) -> ChannelReceiver<EngineEvent>;
}
