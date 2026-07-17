mod automation;
mod capability;
mod components;
mod config;
mod descriptor_validation;
mod dynamic_event_abi;
mod dynamic_events;
mod engine;
mod kira_bridge;
mod mixer_configuration;
mod module;
mod output;
mod package;
mod plugin;
mod poison_recovery;
mod presets;
mod ray_tracing;
mod runtime_plugin;
mod service_types;
mod timeline;

pub use capability::{
    PLUGIN_ID, RUNTIME_CAPABILITIES, SOUND_RAY_TRACED_CONVOLUTION_REVERB_CAPABILITY,
    SOUND_RUNTIME_CAPABILITY, SOUND_TIMELINE_ANIMATION_TRACK_CAPABILITY,
};
pub use components::sound_component_descriptors;
pub use config::SoundConfig;
pub use module::{
    module_descriptor, SoundModule, SOUND_DRIVER_NAME, SOUND_MANAGER_NAME, SOUND_MODULE_NAME,
};
pub use package::dependencies::sound_dependencies;
pub use package::events::{sound_event_catalogs, SOUND_DYNAMIC_EVENT_NAMESPACE};
pub use package::options::sound_options;
pub use plugin::{
    package_manifest, plugin_registration, primary_runtime_capability, runtime_capabilities,
    runtime_plugin, runtime_selection, sound_dist_module_manifest, SoundRuntimePlugin,
    SOUND_DIST_CRATE_NAME, SOUND_DIST_RUNTIME_ENTRY,
};
pub use runtime_plugin::descriptor::runtime_plugin_descriptor;
pub use runtime_plugin::feature_manifest::{
    sound_ray_traced_convolution_reverb_feature_manifest,
    sound_timeline_animation_track_feature_manifest,
};
pub use service_types::{DefaultSoundManager, SoundDriver};

#[cfg(test)]
mod tests;
