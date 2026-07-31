//! Stable engine-facing manager handles, service names, and resolver helpers.

mod resolver;
mod service;
mod service_names;

#[cfg(feature = "ai-contracts")]
pub use resolver::ai_manager_handle;
#[cfg(feature = "net-contracts")]
pub use resolver::net_manager_handle;
#[cfg(feature = "physics-contracts")]
pub use resolver::physics_manager_handle;
#[cfg(feature = "sound-contracts")]
pub use resolver::sound_manager_handle;
pub use resolver::{
    animation_manager_handle, config_manager_handle, event_manager_handle,
    input_action_manager_handle, input_manager_handle, level_manager_handle,
    navigation_manager_handle, platform_preference_storage_handle, render_framework_handle,
    rendering_manager_handle, resource_manager_handle, ManagerResolver,
};
pub use service::{
    manager_service_handle, resolve_manager_service, ManagerServiceHandle, ManagerServiceResolver,
    RegisteredManagerService,
};
#[cfg(feature = "ai-contracts")]
pub use service_names::AI_MANAGER_NAME;
#[cfg(feature = "net-contracts")]
pub use service_names::NET_MANAGER_NAME;
#[cfg(feature = "physics-contracts")]
pub use service_names::PHYSICS_MANAGER_NAME;
#[cfg(feature = "sound-contracts")]
pub use service_names::SOUND_MANAGER_NAME;
pub use service_names::{
    ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_ACTION_MANAGER_NAME,
    INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME, NAVIGATION_MANAGER_NAME, PLATFORM_MANAGER_NAME,
    RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME, RESOURCE_MANAGER_NAME,
};

#[cfg(test)]
mod tests;
