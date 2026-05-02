pub const FEATURE_ID: &str = "sound.timeline_animation_track";

#[derive(Clone, Debug)]
pub struct SoundTimelineAnimationRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for SoundTimelineAnimationRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(zircon_runtime::core::ModuleDescriptor::new(
            "SoundTimelineAnimationFeatureModule",
            "Sound timeline animation trigger track feature",
        ))
    }
}

pub fn runtime_plugin_feature() -> SoundTimelineAnimationRuntimeFeature {
    SoundTimelineAnimationRuntimeFeature
}

pub fn plugin_feature_registration() -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(&runtime_plugin_feature())
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        FEATURE_ID,
        "Sound Timeline Animation Track",
        "sound",
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        "sound",
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
