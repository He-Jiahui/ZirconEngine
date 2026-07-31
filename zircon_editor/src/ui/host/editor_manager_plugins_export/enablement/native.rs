use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::native::NativePluginLoader;

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::editor_capabilities_for_package;
use super::super::reports::EditorPluginEnableReport;

impl EditorManager {
    pub fn set_native_aware_project_plugin_enabled(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorPluginEnableReport, String> {
        let project_root = project_root.as_ref();
        let native_report = NativePluginLoader.discover(self.plugin_directory(project_root));
        if self
            .runtime_plugin_catalog()
            .project_selection_for_package(plugin_id)
            .is_some()
        {
            let report =
                self.set_project_plugin_enabled_unpublished(manifest, plugin_id, enabled)?;
            let completed =
                self.complete_project_plugin_manifest_with_native_report(manifest, &native_report);
            self.publish_project_plugin_status_from_load_report(&completed, &native_report);
            return Ok(report);
        }

        let native_projection = native_report.projection();
        let native_package = native_projection
            .package_manifests()
            .iter()
            .find(|package| package.id == plugin_id)
            .cloned();
        let mut completed =
            self.complete_project_plugin_manifest_with_native_report(manifest, &native_report);
        let mut selection = completed
            .plugins
            .selections
            .iter()
            .find(|selection| selection.id == plugin_id)
            .cloned()
            .ok_or_else(|| {
                format!("plugin {plugin_id} is not registered in builtin or native catalog")
            })?;
        if !enabled && selection.required {
            return Err(format!("required plugin {plugin_id} cannot be disabled"));
        }
        selection.enabled = enabled;
        completed.plugins.set_enabled(selection.clone());
        let editor_capabilities = native_package
            .as_ref()
            .map(editor_capabilities_for_package)
            .unwrap_or_default();
        let capability_snapshot = if editor_capabilities.is_empty() {
            self.capability_snapshot()
        } else {
            self.set_editor_capabilities_enabled(&editor_capabilities, enabled)?
        };
        manifest.plugins.set_enabled(selection.clone());

        let diagnostics = if editor_capabilities.is_empty() {
            vec![format!(
                "native plugin {plugin_id} project selection updated; no editor capabilities were declared"
            )]
        } else {
            vec![format!(
                "native plugin {plugin_id} project selection and editor capabilities updated"
            )]
        };

        let report = EditorPluginEnableReport {
            plugin_id: plugin_id.to_string(),
            enabled,
            project_selection: selection,
            editor_capabilities,
            capability_snapshot,
            diagnostics,
        };
        self.publish_project_plugin_status_from_load_report(&completed, &native_report);
        Ok(report)
    }
}
