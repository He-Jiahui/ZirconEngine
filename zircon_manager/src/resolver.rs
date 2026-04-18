use std::fmt;
use std::sync::Arc;

use zircon_core::{CoreError, CoreHandle};
use zircon_framework::{
    asset::ResourceManager,
    foundation::{ConfigManager, EventManager},
    input::InputManager,
    render::{RenderFramework, RenderingManager},
    scene::LevelManager,
};

use crate::{
    CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME, RESOURCE_MANAGER_NAME,
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
            core.resolve_manager::<$holder>($service_name)
                .map(|holder| holder.shared())
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
