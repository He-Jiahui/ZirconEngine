pub const FEATURE_ID: &str = "sound.timeline_animation_track";
pub const RUNTIME_CAPABILITY: &str = "runtime.feature.sound.timeline_animation_track";
pub const EDITOR_CAPABILITY: &str = "editor.feature.sound.timeline_animation_track";

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

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
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
    .with_capability(RUNTIME_CAPABILITY)
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([RUNTIME_CAPABILITY.to_string()]),
    )
    .with_editor_module(
        zircon_runtime::plugin::PluginModuleManifest::editor(
            "sound.timeline_animation_track.editor",
            "zircon_plugin_sound_timeline_animation_editor",
        )
        .with_capabilities([EDITOR_CAPABILITY.to_string()]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_feature_provider_manifest_matches_sound_owner_contract() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert_eq!(report.manifest.id, FEATURE_ID);
        assert_eq!(
            report.manifest.display_name,
            "Sound Timeline Animation Track"
        );
        assert_eq!(report.manifest.owner_plugin_id, "sound");
        assert!(!report.manifest.enabled_by_default);
        assert_eq!(
            report.manifest.default_packaging,
            vec![
                zircon_runtime::plugin::ExportPackagingStrategy::SourceTemplate,
                zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
            ]
        );
        assert!(report
            .manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(report.manifest.dependencies.iter().any(|dependency| {
            dependency.plugin_id == "sound"
                && dependency.capability == "runtime.plugin.sound"
                && dependency.primary
        }));
        assert!(report.manifest.dependencies.iter().any(|dependency| {
            dependency.plugin_id == "animation"
                && dependency.capability == "runtime.feature.animation.timeline_event_track"
                && !dependency.primary
        }));
        assert!(report.manifest.modules.iter().any(|module| {
            module.name == "sound.timeline_animation_track.runtime"
                && module.crate_name == "zircon_plugin_sound_timeline_animation_runtime"
                && module
                    .capabilities
                    .contains(&RUNTIME_CAPABILITY.to_string())
        }));
        assert!(report.manifest.modules.iter().any(|module| {
            module.name == "sound.timeline_animation_track.editor"
                && module.crate_name == "zircon_plugin_sound_timeline_animation_editor"
                && module.capabilities.contains(&EDITOR_CAPABILITY.to_string())
        }));
    }
}
