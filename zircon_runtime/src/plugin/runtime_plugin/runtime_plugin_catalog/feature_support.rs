use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind};

pub(super) fn owner_dependency_is_valid(feature: &PluginFeatureBundleManifest) -> bool {
    let mut primary_dependencies = feature
        .dependencies
        .iter()
        .filter(|dependency| dependency.primary);
    let Some(primary_dependency) = primary_dependencies.next() else {
        return false;
    };
    primary_dependency.plugin_id == feature.owner_plugin_id && primary_dependencies.next().is_none()
}

pub(super) fn plugin_is_enabled_for_target(
    plugin_id: &str,
    plugin_selections: &HashMap<&str, &ProjectPluginSelection>,
    enabled_plugins: &HashSet<String>,
) -> bool {
    plugin_selections.contains_key(plugin_id) && enabled_plugins.contains(plugin_id)
}

pub(super) fn feature_manifest_supports_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> bool {
    let mut runtime_modules = feature
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime);
    let Some(first_runtime_module) = runtime_modules.next() else {
        return true;
    };
    first_runtime_module.target_modes.is_empty()
        || first_runtime_module.target_modes.contains(&target)
        || runtime_modules
            .any(|module| module.target_modes.is_empty() || module.target_modes.contains(&target))
}

#[cfg(test)]
mod tests {
    use crate::core::framework::platform::RuntimeTargetMode;
    use crate::plugin::{
        PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
    };

    #[test]
    fn feature_support_predicates_do_not_collect_filtered_modules_or_dependencies() {
        let source = include_str!("feature_support.rs");
        let collecting_shape = [".collect::<", "Vec<_>>()"].concat();
        assert!(!source.contains(&collecting_shape));
    }

    #[test]
    fn owner_dependency_validation_requires_one_matching_primary() {
        let valid = PluginFeatureBundleManifest::new("rendering.ssao", "SSAO", "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_dependency(PluginFeatureDependency::required(
                "render_graph",
                "runtime.module.render_graph",
            ));
        assert!(super::owner_dependency_is_valid(&valid));

        let duplicate_primary = valid
            .clone()
            .with_dependency(PluginFeatureDependency::primary(
                "render_graph",
                "runtime.module.render_graph",
            ));
        assert!(!super::owner_dependency_is_valid(&duplicate_primary));
        assert!(!super::owner_dependency_is_valid(
            &PluginFeatureBundleManifest::new("rendering.ssao", "SSAO", "rendering")
        ));
    }

    #[test]
    fn target_support_streams_runtime_modules() {
        let metadata_only = PluginFeatureBundleManifest::new("rendering.ssao", "SSAO", "rendering");
        assert!(super::feature_manifest_supports_target(
            &metadata_only,
            RuntimeTargetMode::ClientRuntime
        ));

        let editor_only = metadata_only.with_runtime_module(
            PluginModuleManifest::runtime("rendering.ssao.runtime", "ssao_runtime")
                .with_target_modes([RuntimeTargetMode::EditorHost]),
        );
        assert!(super::feature_manifest_supports_target(
            &editor_only,
            RuntimeTargetMode::EditorHost
        ));
        assert!(!super::feature_manifest_supports_target(
            &editor_only,
            RuntimeTargetMode::ClientRuntime
        ));
    }
}
