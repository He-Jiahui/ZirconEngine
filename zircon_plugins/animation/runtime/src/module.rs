use std::sync::Arc;

use zircon_runtime::core::framework::animation::AnimationManager;
use zircon_runtime::core::manager::RegisteredManagerService;
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use zircon_runtime::engine_module::{EngineModule, dependency_on, factory, qualified_name};

use crate::DefaultAnimationManager;

pub const ANIMATION_MODULE_NAME: &str = "animation.runtime";
pub const ANIMATION_DRIVER_NAME: &str = "animation.runtime.Driver.AnimationDriver";
pub const DEFAULT_ANIMATION_MANAGER_NAME: &str =
    "animation.runtime.Manager.DefaultAnimationManager";

#[derive(Clone, Debug, Default)]
pub struct AnimationDriver;

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
        factory(|core| Ok(Arc::new(DefaultAnimationManager::new(Some(core))) as ServiceObject)),
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
            Ok(
                Arc::new(RegisteredManagerService::<dyn AnimationManager>::new(
                    manager,
                )) as ServiceObject,
            )
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
