use zircon_module::{EngineModule, ModuleDescriptor};

pub use zircon_animation::{AnimationDriver, AnimationManager};

pub const ANIMATION_MODULE_NAME: &str = "AnimationModule";
pub const ANIMATION_DRIVER_NAME: &str = "AnimationModule.Driver.AnimationDriver";
pub const ANIMATION_MANAGER_NAME: &str = "AnimationModule.Manager.AnimationManager";

#[derive(Clone, Debug, Default)]
pub struct AnimationConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationModule;

pub fn module_descriptor() -> ModuleDescriptor {
    super::module_descriptor_with_driver_and_manager::<AnimationDriver, AnimationManager>(
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
