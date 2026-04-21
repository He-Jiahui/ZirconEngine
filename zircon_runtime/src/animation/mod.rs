mod animation_interface;
mod config;
mod module;
mod sequence_runtime;
mod service_types;

pub use animation_interface::AnimationInterface;
pub use config::AnimationConfig;
pub use module::{
    module_descriptor, AnimationModule, ANIMATION_DRIVER_NAME, ANIMATION_MANAGER_NAME,
    ANIMATION_MODULE_NAME, ANIMATION_PLAYBACK_CONFIG_KEY,
};
pub use sequence_runtime::{apply_sequence_to_world, AnimationSequenceApplyReport};
pub use service_types::{AnimationDriver, DefaultAnimationManager};

#[cfg(test)]
pub(crate) use module::DEFAULT_ANIMATION_MANAGER_NAME;

#[cfg(test)]
mod tests;
