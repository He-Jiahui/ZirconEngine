mod service_types;

use crate::engine_module::{EngineModule, ModuleDescriptor};

pub use service_types::{SoundDriver, SoundManager};

pub const SOUND_MODULE_NAME: &str = "SoundModule";
pub const SOUND_DRIVER_NAME: &str = "SoundModule.Driver.SoundDriver";
pub const SOUND_MANAGER_NAME: &str = "SoundModule.Manager.SoundManager";

#[derive(Clone, Debug, Default)]
pub struct SoundConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SoundModule;

pub fn module_descriptor() -> ModuleDescriptor {
    super::module_descriptor_with_driver_and_manager::<SoundDriver, SoundManager>(
        SOUND_MODULE_NAME,
        "Audio mixing, buses, and playback",
        "SoundDriver",
        "SoundManager",
    )
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
