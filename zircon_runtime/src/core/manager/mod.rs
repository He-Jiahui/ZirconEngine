//! Stable engine-facing manager handles, service names, and resolver helpers.

mod resolver;
mod service_names;

#[cfg(feature = "ai-contracts")]
pub use crate::core::framework::ai::AiManager;
pub use crate::core::framework::animation::AnimationManager;
pub use crate::core::framework::asset::ResourceManager;
pub use crate::core::framework::foundation::{ConfigManager, EventManager};
pub use crate::core::framework::input::{InputActionManager, InputManager};
pub use crate::core::framework::navigation::NavigationManager;
#[cfg(feature = "net-contracts")]
pub use crate::core::framework::net::NetManager;
#[cfg(feature = "physics-contracts")]
pub use crate::core::framework::physics::PhysicsManager;
pub use crate::core::framework::render::{RenderingBackendInfo, RenderingManager};
#[cfg(feature = "sound-contracts")]
pub use crate::core::framework::sound::SoundManager;
#[cfg(feature = "ai-contracts")]
pub use resolver::{resolve_ai_manager, AiManagerHandle};
pub use resolver::{
    resolve_animation_manager, resolve_config_manager, resolve_event_manager,
    resolve_input_action_manager, resolve_input_manager, resolve_level_manager,
    resolve_navigation_manager, resolve_render_framework, resolve_rendering_manager,
    resolve_resource_manager, AnimationManagerHandle, ConfigManagerHandle, EventManagerHandle,
    InputActionManagerHandle, InputManagerHandle, LevelManagerHandle, ManagerResolver,
    NavigationManagerHandle, RenderFrameworkHandle, RenderingManagerHandle, ResourceManagerHandle,
};
#[cfg(feature = "net-contracts")]
pub use resolver::{resolve_net_manager, NetManagerHandle};
#[cfg(feature = "physics-contracts")]
pub use resolver::{resolve_physics_manager, PhysicsManagerHandle};
#[cfg(feature = "sound-contracts")]
pub use resolver::{resolve_sound_manager, SoundManagerHandle};
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
    INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME, NAVIGATION_MANAGER_NAME, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RESOURCE_MANAGER_NAME,
};

#[cfg(test)]
mod tests;
