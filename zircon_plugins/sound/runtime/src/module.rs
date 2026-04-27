use std::sync::Arc;

use zircon_runtime::asset::ASSET_MODULE_NAME;
use zircon_runtime::core::manager::SoundManagerHandle;
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name, EngineModule};

use super::{DefaultSoundManager, SoundDriver};

pub const SOUND_MODULE_NAME: &str = "SoundModule";
pub const SOUND_DRIVER_NAME: &str = "SoundModule.Driver.SoundDriver";
pub(crate) const DEFAULT_SOUND_MANAGER_NAME: &str = "SoundModule.Manager.DefaultSoundManager";
pub const SOUND_MANAGER_NAME: &str = zircon_runtime::core::manager::SOUND_MANAGER_NAME;

#[derive(Clone, Copy, Debug, Default)]
pub struct SoundModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(SOUND_MODULE_NAME, "Audio mixing, buses, and playback")
        .with_driver(DriverDescriptor::new(
            qualified_name(SOUND_MODULE_NAME, ServiceKind::Driver, "SoundDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(SoundDriver) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(
                SOUND_MODULE_NAME,
                ServiceKind::Manager,
                "DefaultSoundManager",
            ),
            StartupMode::Lazy,
            vec![
                dependency_on(SOUND_MODULE_NAME, ServiceKind::Driver, "SoundDriver"),
                dependency_on(
                    ASSET_MODULE_NAME,
                    ServiceKind::Manager,
                    "ProjectAssetManager",
                ),
            ],
            factory(|core| {
                Ok(Arc::new(DefaultSoundManager::new(Some(core.clone()))) as ServiceObject)
            }),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(SOUND_MODULE_NAME, ServiceKind::Manager, "SoundManager"),
            StartupMode::Lazy,
            vec![dependency_on(
                SOUND_MODULE_NAME,
                ServiceKind::Manager,
                "DefaultSoundManager",
            )],
            factory(|core| {
                let manager =
                    core.resolve_manager::<DefaultSoundManager>(DEFAULT_SOUND_MANAGER_NAME)?;
                Ok(Arc::new(SoundManagerHandle::new(manager)) as ServiceObject)
            }),
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
