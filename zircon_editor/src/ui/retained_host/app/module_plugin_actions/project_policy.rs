use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::ExportPackagingStrategy;

pub(super) fn feature_dependency_enable_message(
    report: &crate::ui::host::EditorPluginFeatureSelectionUpdateReport,
) -> String {
    let mut details = Vec::new();
    if !report.enabled_dependency_plugins.is_empty() {
        details.push(format!(
            "plugins {}",
            report.enabled_dependency_plugins.join(", ")
        ));
    }
    if !report.enabled_dependency_features.is_empty() {
        details.push(format!(
            "features {}",
            report.enabled_dependency_features.join(", ")
        ));
    }
    if details.is_empty() {
        let mut message = format!("Feature {} dependencies already enabled", report.feature_id);
        if !report.diagnostics.is_empty() {
            message.push_str(": ");
            message.push_str(&report.diagnostics.join("; "));
        }
        return message;
    }
    let mut message = format!(
        "Feature {} dependencies enabled: {}",
        report.feature_id,
        details.join("; ")
    );
    if !report.diagnostics.is_empty() {
        message.push_str("; ");
        message.push_str(&report.diagnostics.join("; "));
    }
    message
}

pub(super) fn current_native_aware_project_selection(
    editor_manager: &crate::ui::host::EditorManager,
    project_root: &std::path::Path,
    manifest: &ProjectManifest,
    plugin_id: &str,
) -> Result<zircon_runtime::plugin::ProjectPluginSelection, String> {
    editor_manager
        .complete_native_aware_project_plugin_manifest(project_root, manifest)
        .plugins
        .selections
        .into_iter()
        .find(|selection| selection.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} is not registered in builtin or native catalog"))
}

pub(super) fn next_packaging(packaging: ExportPackagingStrategy) -> ExportPackagingStrategy {
    match packaging {
        ExportPackagingStrategy::LibraryEmbed => ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::NativeDynamic => ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::SourceTemplate => ExportPackagingStrategy::LibraryEmbed,
    }
}

pub(super) fn next_target_modes(target_modes: &[RuntimeTargetMode]) -> Vec<RuntimeTargetMode> {
    match target_modes {
        [] => vec![RuntimeTargetMode::ClientRuntime],
        [RuntimeTargetMode::ClientRuntime] => vec![RuntimeTargetMode::ServerRuntime],
        [RuntimeTargetMode::ServerRuntime] => vec![RuntimeTargetMode::EditorHost],
        [RuntimeTargetMode::EditorHost] => {
            vec![
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ]
        }
        [RuntimeTargetMode::ClientRuntime, RuntimeTargetMode::EditorHost] => Vec::new(),
        _ => Vec::new(),
    }
}

pub(super) fn packaging_status_label(packaging: ExportPackagingStrategy) -> &'static str {
    match packaging {
        ExportPackagingStrategy::SourceTemplate => "source-template",
        ExportPackagingStrategy::LibraryEmbed => "library-embed",
        ExportPackagingStrategy::NativeDynamic => "native-dynamic",
    }
}

pub(super) fn target_modes_status_label(target_modes: &[RuntimeTargetMode]) -> String {
    if target_modes.is_empty() {
        return "all".to_string();
    }
    target_modes
        .iter()
        .map(|mode| match mode {
            RuntimeTargetMode::ClientRuntime => "client",
            RuntimeTargetMode::ServerRuntime => "server",
            RuntimeTargetMode::EditorHost => "editor",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

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
            project_selection: zircon_runtime::plugin::ProjectPluginSelection::runtime_plugin(
                zircon_runtime::builtin::RuntimePluginId::Rendering,
                true,
                false,
            ),
            enabled_dependency_plugins: vec!["rendering".to_string(), "particles".to_string()],
            enabled_dependency_features: vec!["rendering.shader_graph".to_string()],
            diagnostics: vec![
                "enabled dependencies for feature rendering.vfx_graph on plugin rendering"
                    .to_string(),
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
}
