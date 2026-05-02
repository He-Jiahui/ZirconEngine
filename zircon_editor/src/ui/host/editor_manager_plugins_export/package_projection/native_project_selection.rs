use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::PluginFeatureBundleManifest, plugin::PluginModuleKind,
    plugin::PluginPackageManifest, plugin::ProjectPluginFeatureSelection,
    plugin::ProjectPluginSelection, RuntimeTargetMode,
};

use super::module_crate_lookup::module_crate;

pub(in crate::ui::host::editor_manager_plugins_export) fn native_project_selection(
    package: &PluginPackageManifest,
) -> ProjectPluginSelection {
    let mut target_modes = Vec::new();
    for target_mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    ProjectPluginSelection {
        id: package.id.clone(),
        enabled: false,
        required: false,
        target_modes,
        packaging: zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
        runtime_crate: module_crate(package, PluginModuleKind::Runtime),
        editor_crate: module_crate(package, PluginModuleKind::Editor),
        features: package
            .optional_features
            .iter()
            .map(native_project_feature_selection)
            .collect(),
    }
}

fn native_project_feature_selection(
    feature: &PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut selection = ProjectPluginFeatureSelection::new(feature.id.clone())
        .enabled(feature.enabled_by_default)
        .with_packaging(default_feature_packaging(feature))
        .with_target_modes(feature_target_modes(feature));
    if let Some(crate_name) = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .map(|module| module.crate_name.clone())
    {
        selection = selection.with_runtime_crate(crate_name);
    }
    if let Some(crate_name) = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Editor)
        .map(|module| module.crate_name.clone())
    {
        selection = selection.with_editor_crate(crate_name);
    }
    selection
}

fn default_feature_packaging(feature: &PluginFeatureBundleManifest) -> ExportPackagingStrategy {
    feature
        .default_packaging
        .iter()
        .copied()
        .find(|packaging| *packaging == ExportPackagingStrategy::NativeDynamic)
        .or_else(|| feature.default_packaging.first().copied())
        .unwrap_or(ExportPackagingStrategy::NativeDynamic)
}

fn feature_target_modes(feature: &PluginFeatureBundleManifest) -> Vec<RuntimeTargetMode> {
    let mut target_modes = Vec::new();
    for target_mode in feature
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    target_modes
}

#[cfg(test)]
mod tests {
    use zircon_runtime::{
        plugin::PluginFeatureBundleManifest, plugin::PluginFeatureDependency,
        plugin::PluginModuleManifest, RuntimeTargetMode,
    };

    use super::*;

    #[test]
    fn native_selection_aggregates_runtime_and_editor_module_target_modes() {
        let package = PluginPackageManifest::new("split_target_tool", "Split Target Tool")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "split_target_tool.runtime",
                    "zircon_plugin_split_target_tool_runtime",
                )
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
            )
            .with_editor_module(
                PluginModuleManifest::editor(
                    "split_target_tool.editor",
                    "zircon_plugin_split_target_tool_editor",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost]),
            );

        let selection = native_project_selection(&package);

        assert_eq!(
            selection.target_modes,
            vec![
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost
            ]
        );
    }

    #[test]
    fn native_selection_preserves_optional_feature_defaults() {
        let package = PluginPackageManifest::new("native_tool", "Native Tool")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "native_tool.runtime",
                    "zircon_plugin_native_tool_runtime",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost]),
            )
            .with_optional_feature(
                PluginFeatureBundleManifest::new(
                    "native_tool.timeline_bridge",
                    "Native Timeline Bridge",
                    "native_tool",
                )
                .with_dependency(PluginFeatureDependency::primary(
                    "native_tool",
                    "runtime.plugin.native_tool",
                ))
                .with_default_packaging([ExportPackagingStrategy::NativeDynamic])
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        "native_tool.timeline_bridge.runtime",
                        "zircon_plugin_native_tool_timeline_bridge_runtime",
                    )
                    .with_target_modes([RuntimeTargetMode::EditorHost]),
                ),
            );

        let selection = native_project_selection(&package);

        assert_eq!(selection.features.len(), 1);
        assert_eq!(selection.features[0].id, "native_tool.timeline_bridge");
        assert!(!selection.features[0].enabled);
        assert_eq!(
            selection.features[0].packaging,
            ExportPackagingStrategy::NativeDynamic
        );
        assert_eq!(
            selection.features[0].runtime_crate.as_deref(),
            Some("zircon_plugin_native_tool_timeline_bridge_runtime")
        );
        assert_eq!(
            selection.features[0].target_modes,
            vec![RuntimeTargetMode::EditorHost]
        );
    }
}
