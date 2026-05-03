pub const PLUGIN_ID: &str = "sound";

mod automation;
mod components;
mod config;
mod descriptor_validation;
mod dynamic_events;
mod engine;
mod mixer_configuration;
mod module;
mod output;
mod package;
mod presets;
mod ray_tracing;
mod service_types;

pub use components::sound_component_descriptors;
pub use config::SoundConfig;
pub use module::{
    module_descriptor, SoundModule, SOUND_DRIVER_NAME, SOUND_MANAGER_NAME, SOUND_MODULE_NAME,
};
pub use package::{
    sound_dependencies, sound_event_catalogs, sound_options, SOUND_DYNAMIC_EVENT_NAMESPACE,
};
pub use service_types::{DefaultSoundManager, SoundDriver};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct SoundRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl SoundRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for SoundRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> zircon_runtime::plugin::PluginPackageManifest {
        package::attach_sound_manifest_contributions(self.descriptor.package_manifest())
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        for component in sound_component_descriptors() {
            registry.register_component(component)?;
        }
        for option in sound_options() {
            registry.register_plugin_option(option)?;
        }
        for event_catalog in sound_event_catalogs() {
            registry.register_plugin_event_catalog(event_catalog)?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Sound",
        zircon_runtime::RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound")
    .with_optional_feature(sound_timeline_animation_track_feature_manifest())
    .with_optional_feature(sound_ray_traced_convolution_reverb_feature_manifest())
}

pub fn runtime_plugin() -> SoundRuntimePlugin {
    SoundRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.sound"]
}

pub fn sound_timeline_animation_track_feature_manifest(
) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        "runtime.plugin.sound",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "animation",
        "runtime.feature.animation.timeline_event_track",
    ))
    .with_capability("runtime.feature.sound.timeline_animation_track")
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.timeline_animation_track".to_string()]),
    )
    .with_editor_module(zircon_runtime::plugin::PluginModuleManifest::editor(
        "sound.timeline_animation_track.editor",
        "zircon_plugin_sound_timeline_animation_editor",
    ))
}

pub fn sound_ray_traced_convolution_reverb_feature_manifest(
) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        "sound.ray_traced_convolution_reverb",
        "Ray Traced Convolution Reverb",
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        "runtime.plugin.sound",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.plugin.physics",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.capability.physics.raycast",
    ))
    .with_capability("runtime.feature.sound.ray_traced_convolution_reverb")
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "sound.ray_traced_convolution_reverb.runtime",
            "zircon_plugin_sound_ray_traced_convolution_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.ray_traced_convolution_reverb".to_string()]),
    )
    .with_editor_module(zircon_runtime::plugin::PluginModuleManifest::editor(
        "sound.ray_traced_convolution_reverb.editor",
        "zircon_plugin_sound_ray_traced_convolution_editor",
    ))
}
