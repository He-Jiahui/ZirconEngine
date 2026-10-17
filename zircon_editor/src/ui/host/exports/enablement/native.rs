use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::native::discovery::discover_native_plugins;
use zircon_runtime::plugin::PluginPackageManifest;

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::editor_capabilities_for_package;
use super::super::reports::EditorPluginEnableReport;

impl EditorManager {
    /// Prepare catalog selection without changing capabilities, lifecycle, or the published Arc.
    pub(super) fn prepare_durable_plugin_selection(
        &self,
        project_root: &Path,
        manifest: &ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<super::transaction::PreparedPluginSelection, String> {
        let native_report = discover_native_plugins(self.plugin_directory(project_root));
        let mut candidate =
            self.complete_project_plugin_manifest_with_native_report(manifest, &native_report);
        let previous_enabled = candidate
            .plugins
            .selections
            .iter()
            .find(|p| p.id == plugin_id)
            .map(|p| p.enabled)
            .unwrap_or(false);
        let owns_editor_state = self.editor_plugin_catalog().package(plugin_id).is_some();
        // Project/native catalog rows have bookkeeping state, not a proven Rust lifecycle handle.
        let live_editor_owner = self
            .plugin_manager()
            .state_snapshot()
            .entry(plugin_id)
            .is_some_and(|entry| {
                matches!(
                    entry.source(),
                    crate::core::plugin::EditorPluginSource::Builtin
                )
            });
        let (selection, capabilities, live_supported) = if self
            .runtime_plugin_catalog()
            .project_selection_for_package(plugin_id)
            .is_some()
            || owns_editor_state
        {
            let (_, selection, capabilities) =
                self.prepare_project_plugin_selection(&candidate, plugin_id, enabled)?;
            if owns_editor_state {
                self.validate_editor_plugin_state(plugin_id, enabled)?;
            }
            (selection, capabilities, live_editor_owner)
        } else {
            let projection = native_report.projection();
            let package = product_native_package(projection.package_manifests(), plugin_id)
                .ok_or_else(|| {
                    format!("native plugin {plugin_id} has no product-eligible package to enable")
                })?;
            let mut selection = candidate
                .plugins
                .selections
                .iter()
                .find(|p| p.id == plugin_id)
                .cloned()
                .ok_or_else(|| {
                    format!("plugin {plugin_id} is not registered in builtin or native catalog")
                })?;
            if !enabled && selection.required {
                return Err(format!("required plugin {plugin_id} cannot be disabled"));
            }
            selection.enabled = enabled;
            (selection, editor_capabilities_for_package(package), false)
        };
        candidate.plugins.set_enabled(selection.clone());
        Ok(super::transaction::PreparedPluginSelection {
            candidate,
            selection,
            capabilities,
            previous_enabled,
            owns_editor_state,
            live_supported,
            native_report,
        })
    }

    pub fn set_native_aware_project_plugin_enabled(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorPluginEnableReport, String> {
        let project_root = project_root.as_ref();
        let native_report = discover_native_plugins(self.plugin_directory(project_root));
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
        let native_package =
            product_native_package(native_projection.package_manifests(), plugin_id).ok_or_else(
                || format!("native plugin {plugin_id} has no product-eligible package to enable"),
            )?;
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
        let editor_capabilities = editor_capabilities_for_package(native_package);
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

fn product_native_package<'a>(
    packages: &'a [PluginPackageManifest],
    plugin_id: &str,
) -> Option<&'a PluginPackageManifest> {
    packages.iter().find(|package| {
        package.id == plugin_id && package.package_role.is_product_catalog_eligible()
    })
}

#[cfg(test)]
#[path = "tests/native_product_selection.rs"]
mod product_selection_tests;
