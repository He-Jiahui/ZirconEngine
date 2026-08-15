pub const PLUGIN_ID: &str = "animation";
pub const ANIMATION_PLAYBACK_CONFIG_KEY: &str = "animation.playback_settings";

mod clip_event;
mod manager;
mod module;
mod sequence;

pub use clip_event::ProjectAnimationClipEventSampler;
pub use manager::DefaultAnimationManager;
pub use module::{
    module_descriptor, AnimationDriver, AnimationModule, ANIMATION_DRIVER_NAME,
    ANIMATION_MODULE_NAME, DEFAULT_ANIMATION_MANAGER_NAME,
};
pub use sequence::{
    apply_compiled_sequence_to_world, compile_sequence_for_world, CompiledAnimationSequence,
    CompiledAnimationSequenceApplyStats,
};
