//! Stable engine-facing manager handles, service names, and resolver helpers.

mod resolver;
mod service_names;

pub use resolver::{
    resolve_config_manager, resolve_event_manager, resolve_input_manager, resolve_level_manager,
    resolve_render_framework, resolve_rendering_manager, resolve_resource_manager,
    ConfigManagerHandle, EventManagerHandle, InputManagerHandle, LevelManagerHandle,
    ManagerResolver, RenderFrameworkHandle, RenderingManagerHandle, ResourceManagerHandle,
};
pub use zircon_framework::foundation::{ConfigManager, EventManager};
pub use zircon_framework::input::InputManager;
pub use zircon_framework::render::{RenderingBackendInfo, RenderingManager};
pub use zircon_framework::asset::ResourceManager;
pub use service_names::{
    CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};

#[cfg(test)]
mod tests;
