//! Sound module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const SOUND_MODULE_NAME: &str = "SoundModule";

#[derive(Clone, Debug, Default)]
pub struct SoundConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SoundModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
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
