use super::*;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

#[test]
fn module_plugin_policy_cycles_are_deterministic() {
    assert_eq!(
        next_packaging(ExportPackagingStrategy::LibraryEmbed),
        ExportPackagingStrategy::NativeDynamic
    );
    assert_eq!(
        next_packaging(ExportPackagingStrategy::NativeDynamic),
        ExportPackagingStrategy::SourceTemplate
    );
    assert_eq!(
        next_packaging(ExportPackagingStrategy::SourceTemplate),
        ExportPackagingStrategy::LibraryEmbed
    );

    assert_eq!(
        next_target_modes(&[]),
        vec![RuntimeTargetMode::ClientRuntime]
    );
    assert_eq!(
        next_target_modes(&[RuntimeTargetMode::ClientRuntime]),
        vec![RuntimeTargetMode::ServerRuntime]
    );
    assert_eq!(
        next_target_modes(&[RuntimeTargetMode::ServerRuntime]),
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        next_target_modes(&[RuntimeTargetMode::EditorHost]),
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        next_target_modes(&[
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]),
        Vec::<RuntimeTargetMode>::new()
    );
}

#[test]
fn feature_dependency_enable_message_includes_enabled_items_and_diagnostics() {
    let report = crate::ui::host::EditorPluginFeatureSelectionUpdateReport {
        plugin_id: "rendering".to_string(),
        feature_id: "rendering.vfx_graph".to_string(),
        enabled: false,
        project_selection:
            zircon_runtime::core::framework::project::ProjectPluginSelection::runtime_plugin(
                zircon_runtime::builtin::RuntimePluginId::Rendering,
                true,
                false,
            ),
        enabled_dependency_plugins: vec!["rendering".to_string(), "particles".to_string()],
        enabled_dependency_features: vec!["rendering.shader_graph".to_string()],
        diagnostics: vec![
            "enabled dependencies for feature rendering.vfx_graph on plugin rendering".to_string(),
        ],
    };

    assert_eq!(
        feature_dependency_enable_message(&report),
        "Feature rendering.vfx_graph dependencies enabled: plugins rendering, particles; features rendering.shader_graph; enabled dependencies for feature rendering.vfx_graph on plugin rendering"
    );

    let already_ready = crate::ui::host::EditorPluginFeatureSelectionUpdateReport {
        enabled_dependency_plugins: Vec::new(),
        enabled_dependency_features: Vec::new(),
        diagnostics: vec!["dependencies were already enabled".to_string()],
        ..report
    };
    assert_eq!(
        feature_dependency_enable_message(&already_ready),
        "Feature rendering.vfx_graph dependencies already enabled: dependencies were already enabled"
    );
}
