use std::sync::Arc;

use crate::core::manager::AnimationManagerHandle;
use crate::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use crate::engine_module::{dependency_on, factory, qualified_name, EngineModule};

use super::{AnimationDriver, DefaultAnimationManager};

pub const ANIMATION_MODULE_NAME: &str = "AnimationModule";
pub const ANIMATION_DRIVER_NAME: &str = "AnimationModule.Driver.AnimationDriver";
pub(crate) const DEFAULT_ANIMATION_MANAGER_NAME: &str =
    "AnimationModule.Manager.DefaultAnimationManager";
pub const ANIMATION_MANAGER_NAME: &str = crate::core::manager::ANIMATION_MANAGER_NAME;
pub const ANIMATION_PLAYBACK_CONFIG_KEY: &str = "animation.playback_settings";

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        ANIMATION_MODULE_NAME,
        "Animation scheduling and clip playback",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(
            ANIMATION_MODULE_NAME,
            ServiceKind::Driver,
            "AnimationDriver",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(AnimationDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            ANIMATION_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultAnimationManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            Ok(Arc::new(DefaultAnimationManager::new(Some(core.clone()))) as ServiceObject)
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            ANIMATION_MODULE_NAME,
            ServiceKind::Manager,
            "AnimationManager",
        ),
        StartupMode::Immediate,
        vec![dependency_on(
            ANIMATION_MODULE_NAME,
            ServiceKind::Manager,
            "DefaultAnimationManager",
        )],
        factory(|core| {
            let manager =
                core.resolve_manager::<DefaultAnimationManager>(DEFAULT_ANIMATION_MANAGER_NAME)?;
            Ok(Arc::new(AnimationManagerHandle::new(manager)) as ServiceObject)
        }),
    ))
}

impl EngineModule for AnimationModule {
    fn module_name(&self) -> &'static str {
        ANIMATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Animation scheduling and clip playback"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
