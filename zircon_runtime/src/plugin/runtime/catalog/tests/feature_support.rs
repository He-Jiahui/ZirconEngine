use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest};

#[test]
fn feature_support_predicates_do_not_collect_filtered_modules_or_dependencies() {
    let source = include_str!("../feature_support.rs");
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

    let alias_owner =
        PluginFeatureBundleManifest::new("sound.spatial", "Spatial", "sound").with_dependency(
            PluginFeatureDependency::primary("audio", "runtime.plugin.sound"),
        );
    assert!(super::owner_dependency_is_valid(&alias_owner));
}

#[test]
fn feature_owner_enablement_matches_alias_without_duplicating_selection_rows() {
    let selection = crate::core::framework::project::ProjectPluginSelection::runtime_plugin(
        "audio", true, false,
    );
    let plugin_selections = HashMap::from([("audio", &selection)]);
    let selected_plugin_ids = super::canonical_plugin_selection_ids(&plugin_selections);
    let enabled_plugins = HashSet::from(["audio".to_string()]);
    let canonical_enabled_plugins = super::canonical_plugin_ids(&enabled_plugins);

    assert!(super::plugin_is_enabled_for_target(
        "sound",
        &selected_plugin_ids,
        &canonical_enabled_plugins,
    ));
    assert!(!super::plugin_is_enabled_for_target(
        "unknown",
        &selected_plugin_ids,
        &canonical_enabled_plugins,
    ));
    assert!(!super::plugin_is_enabled_for_target(
        "bad/id",
        &selected_plugin_ids,
        &canonical_enabled_plugins,
    ));
    assert!(!super::plugin_is_enabled_for_target(
        "",
        &selected_plugin_ids,
        &canonical_enabled_plugins,
    ));
    assert!(!super::plugin_ids_match("bad/id", "bad/id"));
    assert_eq!(plugin_selections.len(), 1);
}

#[test]
fn feature_owner_enablement_uses_precomputed_canonical_membership() {
    let source = include_str!("../feature_support.rs");
    assert!(source.contains("selected_plugin_ids.contains(canonical_key)"));
    assert!(source.contains("enabled_plugin_ids.contains(canonical_key)"));
    assert!(source.contains("RuntimePluginId::parse_key(plugin_id)"));
    assert!(!source.contains("plugin_selections.keys().any"));
    assert!(!source.contains("enabled_plugins.iter().any"));
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
