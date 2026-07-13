use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest};

use super::rows::RenderingFeatureRow;

const CLIENT_EDITOR_TARGETS: &[RuntimeTargetMode] = &[
    RuntimeTargetMode::ClientRuntime,
    RuntimeTargetMode::EditorHost,
];

pub(super) fn rendering_feature(row: &RenderingFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = format!("rendering.{}", row.id_suffix);
    let capability = format!("runtime.feature.rendering.{}", row.id_suffix);
    let editor_capability = format!("editor.feature.rendering.{}", row.id_suffix);
    let runtime_crate = format!("zircon_plugin_rendering_{}_runtime", row.id_suffix);
    let editor_crate = format!("zircon_plugin_rendering_{}_editor", row.id_suffix);
    let mut manifest =
        PluginFeatureBundleManifest::new(feature_id.clone(), row.display_name, "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_capability(capability.clone())
            .with_runtime_module(
                PluginModuleManifest::runtime(format!("{feature_id}.runtime"), runtime_crate)
                    .with_target_modes(CLIENT_EDITOR_TARGETS.iter().copied())
                    .with_capabilities([capability.clone()]),
            )
            .with_editor_module(
                PluginModuleManifest::editor(format!("{feature_id}.editor"), editor_crate)
                    .with_capabilities([editor_capability]),
            )
            .enabled_by_default(row.enabled_by_default);
    for dependency in row.extra_dependencies {
        manifest = manifest.with_dependency(PluginFeatureDependency::required(
            dependency.provider_plugin_id,
            dependency.capability,
        ));
    }
    manifest
}
