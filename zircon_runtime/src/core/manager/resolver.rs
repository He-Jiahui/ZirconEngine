use std::fmt;
use std::sync::Arc;

#[cfg(feature = "ai-contracts")]
use crate::core::framework::ai::AiManager;
#[cfg(feature = "net-contracts")]
use crate::core::framework::net::NetManager;
#[cfg(feature = "physics-contracts")]
use crate::core::framework::physics::PhysicsManager;
#[cfg(feature = "sound-contracts")]
use crate::core::framework::sound::SoundManager;
use crate::core::framework::{
    animation::AnimationManager,
    asset::ResourceManager,
    foundation::{ConfigManager, EventManager},
    input::{InputActionManager, InputManager},
    navigation::NavigationManager,
    render::{RenderFramework, RenderingManager},
    scene::LevelManager,
};
use crate::core::{CoreError, CoreHandle};

#[cfg(feature = "ai-contracts")]
use super::AI_MANAGER_NAME;
#[cfg(feature = "net-contracts")]
use super::NET_MANAGER_NAME;
#[cfg(feature = "physics-contracts")]
use super::PHYSICS_MANAGER_NAME;
#[cfg(feature = "sound-contracts")]
use super::SOUND_MANAGER_NAME;
use super::{
    ANIMATION_MANAGER_NAME, CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_ACTION_MANAGER_NAME,
    INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME, NAVIGATION_MANAGER_NAME, RENDERING_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RESOURCE_MANAGER_NAME,
};

macro_rules! define_manager_holder {
    ($holder:ident, $trait_name:ident, $resolver:ident, $service_name:ident, $method:ident) => {
        #[derive(Clone)]
        pub struct $holder {
            inner: Arc<dyn $trait_name>,
        }

        impl $holder {
            pub fn new(inner: Arc<dyn $trait_name>) -> Self {
                Self { inner }
            }

            pub fn shared(&self) -> Arc<dyn $trait_name> {
                self.inner.clone()
            }
        }

        impl fmt::Debug for $holder {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($holder)).finish()
            }
        }

        pub fn $resolver(core: &CoreHandle) -> Result<Arc<dyn $trait_name>, CoreError> {
            let holder = core.resolve_manager::<$holder>($service_name)?;
            Ok(holder.shared())
        }

        impl ManagerResolver {
            pub fn $method(&self) -> Result<Arc<dyn $trait_name>, CoreError> {
                $resolver(&self.core)
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct ManagerResolver {
    core: CoreHandle,
}

impl ManagerResolver {
    pub fn new(core: CoreHandle) -> Self {
        Self { core }
    }

    pub fn core(&self) -> &CoreHandle {
        &self.core
    }
}

define_manager_holder!(
    RenderingManagerHandle,
    RenderingManager,
    resolve_rendering_manager,
    RENDERING_MANAGER_NAME,
    rendering
);
define_manager_holder!(
    RenderFrameworkHandle,
    RenderFramework,
    resolve_render_framework,
    RENDER_FRAMEWORK_NAME,
    render_framework
);
define_manager_holder!(
    LevelManagerHandle,
    LevelManager,
    resolve_level_manager,
    LEVEL_MANAGER_NAME,
    level
);
define_manager_holder!(
    ResourceManagerHandle,
    ResourceManager,
    resolve_resource_manager,
    RESOURCE_MANAGER_NAME,
    resource
);
define_manager_holder!(
    InputManagerHandle,
    InputManager,
    resolve_input_manager,
    INPUT_MANAGER_NAME,
    input
);
define_manager_holder!(
    InputActionManagerHandle,
    InputActionManager,
    resolve_input_action_manager,
    INPUT_ACTION_MANAGER_NAME,
    input_actions
);
define_manager_holder!(
    ConfigManagerHandle,
    ConfigManager,
    resolve_config_manager,
    CONFIG_MANAGER_NAME,
    config
);
define_manager_holder!(
    EventManagerHandle,
    EventManager,
    resolve_event_manager,
    EVENT_MANAGER_NAME,
    event
);
#[cfg(feature = "ai-contracts")]
define_manager_holder!(
    AiManagerHandle,
    AiManager,
    resolve_ai_manager,
    AI_MANAGER_NAME,
    ai
);
#[cfg(feature = "net-contracts")]
define_manager_holder!(
    NetManagerHandle,
    NetManager,
    resolve_net_manager,
    NET_MANAGER_NAME,
    net
);
#[cfg(feature = "physics-contracts")]
define_manager_holder!(
    PhysicsManagerHandle,
    PhysicsManager,
    resolve_physics_manager,
    PHYSICS_MANAGER_NAME,
    physics
);
define_manager_holder!(
    AnimationManagerHandle,
    AnimationManager,
    resolve_animation_manager,
    ANIMATION_MANAGER_NAME,
    animation
);
#[cfg(feature = "sound-contracts")]
define_manager_holder!(
    SoundManagerHandle,
    SoundManager,
    resolve_sound_manager,
    SOUND_MANAGER_NAME,
    sound
);
define_manager_holder!(
    NavigationManagerHandle,
    NavigationManager,
    resolve_navigation_manager,
    NAVIGATION_MANAGER_NAME,
    navigation
);
