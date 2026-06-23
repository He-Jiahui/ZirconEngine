pub const PLUGIN_ID: &str = "animation";
mod capability;
mod plugin;
mod runtime_system;

pub use capability::{
    ANIMATION_RUNTIME_CAPABILITY, ANIMATION_TIMELINE_EVENT_TRACK_CAPABILITY, RUNTIME_CAPABILITIES,
};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, AnimationRuntimePlugin, ANIMATION_DIST_CRATE_NAME,
    ANIMATION_DIST_RUNTIME_ENTRY, PLUGIN_RUNTIME_MODULE_NAME,
};
pub use runtime_system::{
    register_runtime_system, AnimationRuntimeSystem, ANIMATION_EVALUATE_SYSTEM,
    ANIMATION_SYSTEM_SET,
};
pub use zircon_runtime::animation::{
    apply_sequence_to_world, module_descriptor, sample_clip_events, AnimationClipEvent,
    AnimationDriver, AnimationModule, DefaultAnimationManager, ANIMATION_DRIVER_NAME,
    ANIMATION_MODULE_NAME, ANIMATION_PLAYBACK_CONFIG_KEY, DEFAULT_ANIMATION_MANAGER_NAME,
};
pub use zircon_runtime::core::framework::animation::AnimationSequenceApplyReport;
pub use zircon_runtime::core::manager::ANIMATION_MANAGER_NAME;

#[cfg(test)]
mod tests;
