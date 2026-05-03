use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::NativePluginLoader;

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
        if self
            .runtime_plugin_catalog()
            .project_selection_for_package(plugin_id)
            .is_some()
        {
            return self.set_project_plugin_enabled(manifest, plugin_id, enabled);
        }

        let project_root = project_root.as_ref();
        let native_packages = NativePluginLoader
            .discover(self.plugin_directory(project_root))
            .package_manifests();
        let native_package = native_packages
            .iter()
            .find(|package| package.id == plugin_id)
            .cloned();
        let completed = self.complete_native_aware_project_plugin_manifest(project_root, manifest);
        let mut selection = completed
            .plugins
            .selections
            .into_iter()
            .find(|selection| selection.id == plugin_id)
            .ok_or_else(|| {
                format!("plugin {plugin_id} is not registered in builtin or native catalog")
            })?;
        if !enabled && selection.required {
            return Err(format!("required plugin {plugin_id} cannot be disabled"));
        }
        selection.enabled = enabled;
        manifest.plugins.set_enabled(selection.clone());
        let editor_capabilities = native_package
            .as_ref()
            .map(editor_capabilities_for_package)
            .unwrap_or_default();
        let capability_snapshot = if editor_capabilities.is_empty() {
            self.capability_snapshot()
        } else {
            self.set_editor_capabilities_enabled(&editor_capabilities, enabled)?
        };

        let diagnostics = if editor_capabilities.is_empty() {
            vec![format!(
                "native plugin {plugin_id} project selection updated; no editor capabilities were declared"
            )]
        } else {
            vec![format!(
                "native plugin {plugin_id} project selection and editor capabilities updated"
            )]
        };

        Ok(EditorPluginEnableReport {
            plugin_id: plugin_id.to_string(),
            enabled,
            project_selection: selection,
            editor_capabilities,
            capability_snapshot,
            diagnostics,
        })
    }
}
