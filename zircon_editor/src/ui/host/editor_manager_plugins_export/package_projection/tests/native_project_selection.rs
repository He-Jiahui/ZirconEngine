use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::{
    plugin::PluginFeatureBundleManifest, plugin::PluginFeatureDependency,
    plugin::PluginModuleManifest,
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
