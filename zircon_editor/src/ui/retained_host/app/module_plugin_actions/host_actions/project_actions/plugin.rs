use std::path::Path;

use super::super::super::super::RetainedEditorHost;
use super::super::super::project_policy::{
    current_native_aware_project_selection, next_packaging, next_target_modes,
    packaging_status_label, target_modes_status_label,
};
use zircon_runtime::asset::project::ProjectManifest;

impl RetainedEditorHost {
    pub(super) fn set_project_plugin_enabled(
        &mut self,
        project_root: &Path,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<String, String> {
        let report = self
            .editor_manager
            .set_native_aware_project_plugin_enabled(project_root, manifest, plugin_id, enabled)?;
        let state = if report.enabled {
            "enabled"
        } else {
            "disabled"
        };
        Ok(format!("Plugin {} {state}", report.plugin_id))
    }

    pub(super) fn cycle_project_plugin_packaging(
        &mut self,
        project_root: &Path,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
    ) -> Result<String, String> {
        let selection = current_native_aware_project_selection(
            &self.editor_manager,
            project_root,
            manifest,
            plugin_id,
        )?;
        let packaging = next_packaging(selection.packaging);
        let report = self
            .editor_manager
            .set_native_aware_project_plugin_packaging(
                project_root,
                manifest,
                plugin_id,
                packaging,
            )?;
        Ok(format!(
            "Plugin {} packaging set to {}",
            report.plugin_id,
            packaging_status_label(report.project_selection.packaging)
        ))
    }

    pub(super) fn cycle_project_plugin_target_modes(
        &mut self,
        project_root: &Path,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
    ) -> Result<String, String> {
        let selection = current_native_aware_project_selection(
            &self.editor_manager,
            project_root,
            manifest,
            plugin_id,
        )?;
        let target_modes = next_target_modes(&selection.target_modes);
        let report = self
            .editor_manager
            .set_native_aware_project_plugin_target_modes(
                project_root,
                manifest,
                plugin_id,
                target_modes,
            )?;
        Ok(format!(
            "Plugin {} target modes set to {}",
            report.plugin_id,
            target_modes_status_label(&report.project_selection.target_modes)
        ))
    }
}
