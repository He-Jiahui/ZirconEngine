//! Sound 插件运行时公共面向框架暴露模块注册、能力清单和经理服务，实际音频资源由内部 Kira 桥接独占。
#[path = "automation/mod.rs"]
mod automation;
#[path = "capability.rs"]
mod capability;
#[path = "components.rs"]
mod components;
#[path = "config.rs"]
mod config;
#[path = "descriptor_validation/mod.rs"]
mod descriptor_validation;
#[path = "dynamic_event_abi/mod.rs"]
mod dynamic_event_abi;
#[path = "dynamic_events/mod.rs"]
mod dynamic_events;
#[path = "engine/mod.rs"]
mod engine;
#[path = "kira_bridge/mod.rs"]
mod kira_bridge;
#[path = "mixer_configuration/mod.rs"]
mod mixer_configuration;
#[path = "module.rs"]
mod module;
#[path = "output/mod.rs"]
mod output;
#[path = "package/mod.rs"]
mod package;
#[path = "plugin.rs"]
mod plugin;
#[path = "poison_recovery.rs"]
mod poison_recovery;
#[path = "presets/mod.rs"]
mod presets;
#[path = "ray_tracing/mod.rs"]
mod ray_tracing;
#[path = "runtime_plugin/mod.rs"]
mod runtime_plugin;
#[path = "scene_system.rs"]
mod scene_system;
#[path = "service_types/mod.rs"]
mod service_types;
#[path = "timeline/mod.rs"]
mod timeline;

pub use capability::{
    NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY,
    NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_ID, RUNTIME_CAPABILITIES, SOUND_DECLARATION,
    SOUND_RAY_TRACED_CONVOLUTION_REVERB_CAPABILITY, SOUND_RUNTIME_CAPABILITY,
    SOUND_TIMELINE_ANIMATION_TRACK_CAPABILITY,
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
#[path = "tests/cases.rs"]
mod tests;
