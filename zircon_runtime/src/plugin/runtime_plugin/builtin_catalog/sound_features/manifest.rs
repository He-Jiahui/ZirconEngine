use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest};

use super::rows::SoundFeatureRow;

pub(super) fn sound_feature(row: &SoundFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = format!("sound.{}", row.id_suffix);
    let mut manifest =
        PluginFeatureBundleManifest::new(feature_id.clone(), row.display_name, "sound")
            .with_dependency(PluginFeatureDependency::primary(
                "sound",
                "runtime.plugin.sound",
            ))
            .with_capability(row.runtime_capability)
            .with_runtime_module(
                PluginModuleManifest::runtime(format!("{feature_id}.runtime"), row.runtime_crate)
                    .with_target_modes(row.runtime_target_modes.iter().copied())
                    .with_capabilities([row.runtime_capability.to_string()]),
            )
            .with_editor_module(
                PluginModuleManifest::editor(format!("{feature_id}.editor"), row.editor_crate)
                    .with_capabilities([row.editor_capability.to_string()]),
            );
    for dependency in row.extra_dependencies {
        manifest = manifest.with_dependency(PluginFeatureDependency::required(
            dependency.provider_plugin_id,
            dependency.capability,
        ));
    }
    manifest
}
