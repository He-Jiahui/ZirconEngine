use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{PluginModuleKind, ProjectPluginSelection};

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::project_selection_from_package;
use super::super::reports::EditorPluginEnableReport;

impl EditorManager {
    pub fn enable_project_plugin(
        &self,
        manifest: &mut ProjectManifest,
        selection: ProjectPluginSelection,
    ) {
        manifest.plugins.set_enabled(selection);
    }

    pub fn disable_project_plugin(&self, manifest: &mut ProjectManifest, plugin_id: &str) {
        manifest.plugins.set_plugin_enabled(plugin_id, false);
    }

    pub fn set_project_plugin_enabled(
        &self,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorPluginEnableReport, String> {
        let runtime_catalog = self.runtime_plugin_catalog();
        let editor_catalog = self.editor_plugin_catalog();
        let editor_packages = editor_catalog.package_manifests();
        let catalog_selection = runtime_catalog
            .project_selection_for_package(plugin_id)
            .or_else(|| {
                editor_packages
                    .iter()
                    .find(|package| package.id == plugin_id)
                    .map(project_selection_from_package)
            })
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
            selection.editor_crate = editor_packages
                .iter()
                .find(|package| package.id == plugin_id)
                .and_then(|package| {
                    package
                        .modules
                        .iter()
                        .find(|module| module.kind == PluginModuleKind::Editor)
                        .map(|module| module.crate_name.clone())
                });
        }
        manifest.plugins.set_enabled(selection.clone());

        let editor_capabilities = editor_catalog.capabilities_for_package(plugin_id);
        let capability_snapshot = if editor_capabilities.is_empty() {
            self.capability_snapshot()
        } else {
            self.set_editor_capabilities_enabled(&editor_capabilities, enabled)?
        };

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
            editor_capabilities,
            capability_snapshot,
            diagnostics,
        })
    }
}
