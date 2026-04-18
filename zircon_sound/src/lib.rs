//! Sound module scaffold with explicit core service descriptors.

use std::sync::Arc;

use zircon_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const SOUND_MODULE_NAME: &str = "SoundModule";
pub const SOUND_DRIVER_NAME: &str = "SoundModule.Driver.SoundDriver";
pub const SOUND_MANAGER_NAME: &str = "SoundModule.Manager.SoundManager";

#[derive(Clone, Debug, Default)]
pub struct SoundConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SoundModule;

#[derive(Clone, Debug, Default)]
pub struct SoundDriver;

#[derive(Clone, Debug, Default)]
pub struct SoundManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(SOUND_MODULE_NAME, "Audio mixing, buses, and playback")
        .with_driver(DriverDescriptor::new(
            qualified_name(SOUND_MODULE_NAME, ServiceKind::Driver, "SoundDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(SoundDriver::default()) as _)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(SOUND_MODULE_NAME, ServiceKind::Manager, "SoundManager"),
            StartupMode::Lazy,
            vec![dependency_on(
                SOUND_MODULE_NAME,
                ServiceKind::Driver,
                "SoundDriver",
            )],
            factory(|_| Ok(Arc::new(SoundManager::default()) as _)),
        ))
}

impl EngineModule for SoundModule {
    fn module_name(&self) -> &'static str {
        SOUND_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Audio mixing, buses, and playback"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
