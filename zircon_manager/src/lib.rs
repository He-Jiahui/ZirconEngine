//! Stable engine-facing manager facades, handles, records, and resolver helpers.

mod builtins;
mod handles;
mod records;
mod resolver;
mod traits;

pub use builtins::{
    module_descriptor, DefaultConfigManager, DefaultEventManager, CONFIG_DRIVER_NAME,
    CONFIG_MANAGER_NAME, EVENT_DRIVER_NAME, EVENT_MANAGER_NAME, MANAGER_MODULE_NAME,
};
pub use handles::{AssetHandle, HostHandle, PluginSlotId, WorldHandle};
pub use records::{
    AssetChangeKind, AssetChangeRecord, AssetPipelineInfo, AssetRecordKind, AssetStatusRecord,
    CapabilitySet, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetChangeKind, EditorAssetChangeRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetReferenceRecord, InputButton, InputEvent, InputEventRecord,
    InputSnapshot, LevelSummary, PreviewStateRecord, ProjectInfo, RenderingBackendInfo,
    ResourceChangeKind, ResourceChangeRecord, ResourceStateRecord, ResourceStatusRecord,
};
pub use resolver::{
    resolve_asset_manager, resolve_config_manager, resolve_event_manager, resolve_input_manager,
    resolve_editor_asset_manager, resolve_level_manager, resolve_rendering_manager,
    resolve_resource_manager, AssetManagerHandle, ConfigManagerHandle, EditorAssetManagerHandle,
    EventManagerHandle, InputManagerHandle, LevelManagerHandle, ManagerResolver,
    RenderingManagerHandle, ResourceManagerHandle,
};
pub use traits::{
    AssetManager, ConfigManager, EditorAssetManager, EventManager, InputManager, LevelManager,
    RenderingManager, ResourceManager,
};

pub const ASSET_MANAGER_NAME: &str = "AssetModule.Manager.AssetManager";
pub const EDITOR_ASSET_MANAGER_NAME: &str = "AssetModule.Manager.EditorAssetManager";
pub const RESOURCE_MANAGER_NAME: &str = "AssetModule.Manager.ResourceManager";
pub const INPUT_MANAGER_NAME: &str = "InputModule.Manager.InputManager";
pub const RENDERING_MANAGER_NAME: &str = "GraphicsModule.Manager.RenderingManager";
pub const LEVEL_MANAGER_NAME: &str = "SceneModule.Manager.LevelManager";

#[cfg(test)]
mod tests;
