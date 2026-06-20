use crate::ui::host::EditorPluginStatusReport;
use crate::ui::workbench::project::project_root_path;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use zircon_runtime::asset::project::ProjectManifest;

use super::super::super::RetainedEditorHost;
use super::super::rows::fallback_project_manifest;

pub(super) struct ModulePluginPaneStatusReport {
    pub(super) report: EditorPluginStatusReport,
    pub(super) diagnostics: Vec<String>,
}

pub(super) fn load_module_plugin_status_report(
    host: &RetainedEditorHost,
    chrome: &EditorChromeSnapshot,
) -> ModulePluginPaneStatusReport {
    let mut diagnostics = Vec::new();
    let report = project_root_path(&chrome.project_path)
        .map_err(|error| error.to_string())
        .and_then(|project_root| {
            let manifest_path = project_root.join("zircon-project.toml");
            ProjectManifest::load(&manifest_path)
                .map(|manifest| {
                    host.editor_manager
                        .native_plugin_status_report(&project_root, &manifest)
                })
                .map_err(|error| {
                    format!(
                        "plugin status uses builtin catalog because project manifest is unavailable: {error}"
                    )
                })
        })
        .unwrap_or_else(|error| {
            diagnostics.push(error);
            host.editor_manager
                .plugin_status_report(&fallback_project_manifest())
        });

    diagnostics.extend(report.diagnostics.iter().cloned());
    ModulePluginPaneStatusReport {
        report,
        diagnostics,
    }
}
