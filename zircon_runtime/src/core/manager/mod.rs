//! Stable engine-facing manager handles, service names, and resolver helpers.

mod resolver;
mod service_names;

pub use resolver::{
    resolve_animation_manager, resolve_config_manager, resolve_event_manager,
    resolve_input_manager, resolve_level_manager, resolve_physics_manager,
    resolve_render_framework, resolve_rendering_manager, resolve_resource_manager,
    AnimationManagerHandle, ConfigManagerHandle, EventManagerHandle, InputManagerHandle,
    LevelManagerHandle, ManagerResolver, PhysicsManagerHandle, RenderFrameworkHandle,
    RenderingManagerHandle, ResourceManagerHandle,
};
pub use crate::core::framework::animation::AnimationManager;
pub use crate::core::framework::asset::ResourceManager;
pub use crate::core::framework::foundation::{ConfigManager, EventManager};
pub use crate::core::framework::input::InputManager;
pub use crate::core::framework::physics::PhysicsManager;
pub use crate::core::framework::render::{RenderingBackendInfo, RenderingManager};
pub use service_names::{
    ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_MANAGER_NAME,
    LEVEL_MANAGER_NAME, PHYSICS_MANAGER_NAME, RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME,
    RESOURCE_MANAGER_NAME,
};

#[cfg(test)]
mod tests;
