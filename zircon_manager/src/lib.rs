//! Stable engine-facing manager facades, handles, records, and resolver helpers.

mod handles;
mod records;
mod resolver;
mod service_names;
mod traits;

pub use handles::{AssetHandle, WorldHandle};
pub use records::{
    AssetChangeKind, AssetChangeRecord, AssetPipelineInfo, AssetStatusRecord, LevelSummary,
    ProjectInfo, RenderingBackendInfo, ResourceChangeKind, ResourceChangeRecord,
    ResourceStateRecord, ResourceStatusRecord,
};
pub use resolver::{
    resolve_asset_manager, resolve_config_manager, resolve_event_manager, resolve_input_manager,
    resolve_level_manager, resolve_rendering_manager, resolve_resource_manager, AssetManagerHandle,
    ConfigManagerHandle, EventManagerHandle, InputManagerHandle, LevelManagerHandle,
    ManagerResolver, RenderingManagerHandle, ResourceManagerHandle,
};
pub use service_names::{
    ASSET_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_MANAGER_NAME,
    LEVEL_MANAGER_NAME, RENDERING_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};
pub use traits::{
    AssetManager, ConfigManager, EventManager, InputManager, LevelManager, RenderingManager,
    ResourceManager,
};

#[cfg(test)]
mod tests;
