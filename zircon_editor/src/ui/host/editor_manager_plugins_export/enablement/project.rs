use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::PluginModuleKind;

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::project_selection_from_package;
use super::super::reports::EditorPluginEnableReport;

impl EditorManager {
    pub fn set_project_plugin_enabled(
        &self,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorPluginEnableReport, String> {
        let report = self.set_project_plugin_enabled_unpublished(manifest, plugin_id, enabled)?;
        self.publish_project_plugin_status(self.plugin_status_report(manifest));
        Ok(report)
    }

    pub(in crate::ui::host) fn set_project_plugin_enabled_unpublished(
        &self,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorPluginEnableReport, String> {
        let runtime_catalog = self.runtime_plugin_catalog();
        let editor_catalog = self.editor_plugin_catalog();
        let editor_package = editor_catalog.package(plugin_id);
        let catalog_selection = runtime_catalog
            .project_selection_for_package(plugin_id)
            .or_else(|| editor_package.map(project_selection_from_package))
            .ok_or_else(|| {
                format!("plugin {plugin_id} is not registered in builtin plugin catalogs")
            })?;
        let existing_selection = manifest
            .plugins
            .selections
            .iter()
            .find(|selection| selection.id == plugin_id)
            .cloned();
        let mut selection = existing_selection.unwrap_or_else(|| catalog_selection.clone());
        if selection.runtime_crate.is_none() {
            selection.runtime_crate = catalog_selection.runtime_crate;
        }
        if selection.editor_crate.is_none() {
            selection.editor_crate = catalog_selection.editor_crate;
        }
        if !enabled && selection.required {
            return Err(format!("required plugin {plugin_id} cannot be disabled"));
        }
        selection.enabled = enabled;
        if selection.editor_crate.is_none() {
            selection.editor_crate = editor_package.and_then(|package| {
                package
                    .modules
                    .iter()
                    .find(|module| module.kind == PluginModuleKind::Editor)
                    .map(|module| module.crate_name.clone())
            });
        }
        let editor_capabilities = editor_catalog.capabilities_for_package(plugin_id);
        let capability_snapshot = if editor_capabilities.is_empty() {
            self.validate_editor_plugin_state(plugin_id, enabled)?;
            self.update_editor_plugin_state_unpublished(plugin_id, enabled)?;
            self.capability_snapshot()
        } else {
            self.set_editor_plugin_enabled_unpublished(plugin_id, enabled)?
        };
        manifest.plugins.set_enabled(selection.clone());

        let mut diagnostics = Vec::new();
        if editor_capabilities.is_empty() {
            diagnostics.push(format!(
                "plugin {plugin_id} has no editor capabilities; project selection updated only"
            ));
        }

        Ok(EditorPluginEnableReport {
            plugin_id: plugin_id.to_string(),
            enabled,
            project_selection: selection,
            editor_capabilities: editor_capabilities.to_vec(),
            capability_snapshot,
            diagnostics,
        })
    }
}
