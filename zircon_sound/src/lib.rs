//! Sound module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const SOUND_MODULE_NAME: &str = "SoundModule";

#[derive(Clone, Debug, Default)]
pub struct SoundConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        SOUND_MODULE_NAME,
        "Audio mixing, buses, and playback",
        "SoundDriver",
        "SoundManager",
    )
}
