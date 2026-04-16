//! Stable engine-facing manager facades, handles, records, and resolver helpers.

mod builtins;
mod handles;
mod records;
mod resolver;
mod service_names;
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
    resolve_asset_manager, resolve_config_manager, resolve_editor_asset_manager,
    resolve_event_manager, resolve_input_manager, resolve_level_manager, resolve_rendering_manager,
    resolve_resource_manager, AssetManagerHandle, ConfigManagerHandle, EditorAssetManagerHandle,
    EventManagerHandle, InputManagerHandle, LevelManagerHandle, ManagerResolver,
    RenderingManagerHandle, ResourceManagerHandle,
};
pub use service_names::{
    ASSET_MANAGER_NAME, EDITOR_ASSET_MANAGER_NAME, INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME,
    RENDERING_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};
pub use traits::{
    AssetManager, ConfigManager, EditorAssetManager, EventManager, InputManager, LevelManager,
    RenderingManager, ResourceManager,
};

#[cfg(test)]
mod tests;
