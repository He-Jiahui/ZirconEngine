//! Animation module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const ANIMATION_MODULE_NAME: &str = "AnimationModule";

#[derive(Clone, Debug, Default)]
pub struct AnimationConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        ANIMATION_MODULE_NAME,
        "Animation scheduling and clip playback",
        "AnimationDriver",
        "AnimationManager",
    )
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
