use serde_json::Value;
use zircon_core::{ChannelReceiver, CoreError, EngineEvent};

use crate::{
    AssetChangeRecord, AssetPipelineInfo, AssetStatusRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetChangeRecord, EditorAssetDetailsRecord, InputEvent, InputEventRecord, InputSnapshot,
    LevelSummary, ProjectInfo, RenderingBackendInfo, ResourceChangeRecord, ResourceStatusRecord,
    WorldHandle,
};

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

pub trait AssetManager: Send + Sync {
    fn pipeline_info(&self) -> AssetPipelineInfo;
    fn open_project(&self, root_path: &str) -> Result<ProjectInfo, CoreError>;
    fn current_project(&self) -> Option<ProjectInfo>;
    fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord>;
    fn list_assets(&self) -> Vec<AssetStatusRecord>;
    fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChangeRecord>;
    fn import_asset(&self, uri: &str) -> Result<Option<AssetStatusRecord>, CoreError>;
    fn reimport_all(&self) -> Result<Vec<AssetStatusRecord>, CoreError>;
}

pub trait EditorAssetManager: Send + Sync {
    fn catalog_snapshot(&self) -> EditorAssetCatalogSnapshotRecord;
    fn asset_details(&self, uuid: &str) -> Option<EditorAssetDetailsRecord>;
    fn subscribe_editor_asset_changes(&self) -> ChannelReceiver<EditorAssetChangeRecord>;
    fn mark_preview_dirty(&self, uuid: &str)
        -> Result<Option<EditorAssetDetailsRecord>, CoreError>;
    fn request_preview_refresh(
        &self,
        uuid: &str,
        visible: bool,
    ) -> Result<Option<EditorAssetDetailsRecord>, CoreError>;
}

pub trait ResourceManager: Send + Sync {
    fn resolve_resource_id(&self, locator: &str) -> Option<String>;
    fn resource_status(&self, locator: &str) -> Option<ResourceStatusRecord>;
    fn list_resources(&self) -> Vec<ResourceStatusRecord>;
    fn resource_revision(&self, locator: &str) -> Option<u64>;
    fn subscribe_resource_changes(&self) -> ChannelReceiver<ResourceChangeRecord>;
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
