use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest};

use super::rows::RenderingFeatureRow;

const CLIENT_EDITOR_TARGETS: &[RuntimeTargetMode] = &[
    RuntimeTargetMode::ClientRuntime,
    RuntimeTargetMode::EditorHost,
];

fn join_string_parts(parts: &[&str]) -> String {
    let capacity = parts.iter().map(|part| part.len()).sum();
    let mut joined = String::with_capacity(capacity);
    for part in parts {
        joined.push_str(part);
    }
    joined
}

pub(super) fn rendering_feature(row: &RenderingFeatureRow) -> PluginFeatureBundleManifest {
    let feature_id = join_string_parts(&["rendering.", row.id_suffix]);
    let capability = join_string_parts(&["runtime.feature.rendering.", row.id_suffix]);
    let editor_capability = join_string_parts(&["editor.feature.rendering.", row.id_suffix]);
    let runtime_crate = join_string_parts(&["zircon_plugin_rendering_", row.id_suffix, "_runtime"]);
    let editor_crate = join_string_parts(&["zircon_plugin_rendering_", row.id_suffix, "_editor"]);
    let mut manifest =
        PluginFeatureBundleManifest::new(feature_id.clone(), row.display_name, "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_capability(capability.clone())
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    join_string_parts(&[&feature_id, ".runtime"]),
                    runtime_crate,
                )
                .with_target_modes(CLIENT_EDITOR_TARGETS.iter().copied())
                .with_capabilities([capability.clone()]),
            )
            .with_editor_module(
                PluginModuleManifest::editor(
                    join_string_parts(&[&feature_id, ".editor"]),
                    editor_crate,
                )
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

#[cfg(test)]
mod tests {
    use super::join_string_parts;

    #[test]
    fn exact_rendering_identifier_join_preserves_parts() {
        assert_eq!(
            join_string_parts(&["runtime.feature.rendering.", "shader_graph"]),
            "runtime.feature.rendering.shader_graph"
        );
    }
}
