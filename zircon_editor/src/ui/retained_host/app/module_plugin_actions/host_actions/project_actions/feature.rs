use std::path::Path;

use super::super::super::super::RetainedEditorHost;
use super::super::super::project_policy::feature_dependency_enable_message;
use zircon_runtime::asset::project::ProjectManifest;

impl RetainedEditorHost {
    pub(super) fn set_project_plugin_feature_enabled(
        &mut self,
        project_root: &Path,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        feature_id: &str,
        enabled: bool,
    ) -> Result<String, String> {
        let report = self
            .editor_manager
            .set_native_aware_project_plugin_feature_enabled(
                project_root,
                manifest,
                plugin_id,
                feature_id,
                enabled,
            )?;
        let state = if report.enabled {
            "enabled"
        } else {
            "disabled"
        };
        Ok(format!(
            "Feature {} on plugin {} {state}",
            report.feature_id, report.plugin_id
        ))
    }

    pub(super) fn enable_project_plugin_feature_dependencies(
        &mut self,
        project_root: &Path,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        feature_id: &str,
    ) -> Result<String, String> {
        let report = self
            .editor_manager
            .enable_native_aware_project_plugin_feature_dependencies(
                project_root,
                manifest,
                plugin_id,
                feature_id,
            )?;
        Ok(feature_dependency_enable_message(&report))
    }
}
