mod capability;
mod plugin;

pub use capability::{
    DIST_CRATE_NAME, DIST_PROVIDER_PACKAGE_ID, DIST_RUNTIME_ENTRY, EDITOR_CAPABILITY, FEATURE_ID,
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY,
    SOUND_RUNTIME_CAPABILITY, SOUND_TIMELINE_ANIMATION_TRACK_DECLARATION,
};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    sound_timeline_animation_dist_module_manifest, SoundTimelineAnimationRuntimeFeature,
};

#[cfg(test)]
mod tests;
