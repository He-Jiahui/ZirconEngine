//! Runtime-owned animation module, manager implementation, and sequence application.

mod module;
pub mod runtime;
pub mod sequence;

pub use module::{
    module_descriptor, AnimationModule, ANIMATION_DRIVER_NAME, ANIMATION_MANAGER_NAME,
    ANIMATION_MODULE_NAME, ANIMATION_PLAYBACK_CONFIG_KEY, DEFAULT_ANIMATION_MANAGER_NAME,
};
pub use runtime::{AnimationDriver, DefaultAnimationManager};
pub use sequence::{apply_sequence_to_world, AnimationSequenceApplyReport};
