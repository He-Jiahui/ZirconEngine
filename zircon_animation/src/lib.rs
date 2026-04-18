//! Animation module scaffold with explicit core service descriptors.

use std::sync::Arc;

use zircon_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const ANIMATION_MODULE_NAME: &str = "AnimationModule";
pub const ANIMATION_DRIVER_NAME: &str = "AnimationModule.Driver.AnimationDriver";
pub const ANIMATION_MANAGER_NAME: &str = "AnimationModule.Manager.AnimationManager";

#[derive(Clone, Debug, Default)]
pub struct AnimationConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationModule;

#[derive(Clone, Debug, Default)]
pub struct AnimationDriver;

#[derive(Clone, Debug, Default)]
pub struct AnimationManager;

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
        factory(|_| Ok(Arc::new(AnimationDriver::default()) as _)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            ANIMATION_MODULE_NAME,
            ServiceKind::Manager,
            "AnimationManager",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            ANIMATION_MODULE_NAME,
            ServiceKind::Driver,
            "AnimationDriver",
        )],
        factory(|_| Ok(Arc::new(AnimationManager::default()) as _)),
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
