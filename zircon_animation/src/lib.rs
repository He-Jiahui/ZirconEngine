//! Animation module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const ANIMATION_MODULE_NAME: &str = "AnimationModule";

#[derive(Clone, Debug, Default)]
pub struct AnimationConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        ANIMATION_MODULE_NAME,
        "Animation scheduling and clip playback",
        "AnimationDriver",
        "AnimationManager",
    )
}
