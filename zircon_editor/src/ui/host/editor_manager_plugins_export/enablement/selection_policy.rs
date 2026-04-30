use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{ExportPackagingStrategy, ProjectPluginSelection, RuntimeTargetMode};

use super::super::super::editor_manager::EditorManager;
use super::super::reports::EditorPluginSelectionUpdateReport;

impl EditorManager {
    pub fn set_project_plugin_packaging(
        &self,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        packaging: ExportPackagingStrategy,
    ) -> Result<EditorPluginSelectionUpdateReport, String> {
        let mut selection = self.completed_builtin_project_selection(manifest, plugin_id)?;
        selection.packaging = packaging;
        manifest.plugins.set_enabled(selection.clone());
        Ok(selection_update_report(
            plugin_id,
            selection,
            format!("plugin {plugin_id} packaging strategy updated"),
        ))
    }

    pub fn set_native_aware_project_plugin_packaging(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        packaging: ExportPackagingStrategy,
    ) -> Result<EditorPluginSelectionUpdateReport, String> {
        let mut selection =
            self.completed_native_aware_project_selection(project_root, manifest, plugin_id)?;
        selection.packaging = packaging;
        manifest.plugins.set_enabled(selection.clone());
        Ok(selection_update_report(
            plugin_id,
            selection,
            format!("plugin {plugin_id} native-aware packaging strategy updated"),
        ))
    }

    pub fn set_project_plugin_target_modes(
        &self,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Result<EditorPluginSelectionUpdateReport, String> {
        let mut selection = self.completed_builtin_project_selection(manifest, plugin_id)?;
        selection.target_modes = deduplicated_target_modes(target_modes);
        manifest.plugins.set_enabled(selection.clone());
        Ok(selection_update_report(
            plugin_id,
            selection,
            format!("plugin {plugin_id} target modes updated"),
        ))
    }

    pub fn set_native_aware_project_plugin_target_modes(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &mut ProjectManifest,
        plugin_id: &str,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Result<EditorPluginSelectionUpdateReport, String> {
        let mut selection =
            self.completed_native_aware_project_selection(project_root, manifest, plugin_id)?;
        selection.target_modes = deduplicated_target_modes(target_modes);
        manifest.plugins.set_enabled(selection.clone());
        Ok(selection_update_report(
            plugin_id,
            selection,
            format!("plugin {plugin_id} native-aware target modes updated"),
        ))
    }

    fn completed_builtin_project_selection(
        &self,
        manifest: &ProjectManifest,
        plugin_id: &str,
    ) -> Result<ProjectPluginSelection, String> {
        self.complete_project_plugin_manifest(manifest)
            .plugins
            .selections
            .into_iter()
            .find(|selection| selection.id == plugin_id)
            .ok_or_else(|| {
                format!("plugin {plugin_id} is not registered in builtin plugin catalogs")
            })
    }

    fn completed_native_aware_project_selection(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        plugin_id: &str,
    ) -> Result<ProjectPluginSelection, String> {
        self.complete_native_aware_project_plugin_manifest(project_root, manifest)
            .plugins
            .selections
            .into_iter()
            .find(|selection| selection.id == plugin_id)
            .ok_or_else(|| {
                format!("plugin {plugin_id} is not registered in builtin or native catalog")
            })
    }
}

fn deduplicated_target_modes(
    target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
) -> Vec<RuntimeTargetMode> {
    let mut deduplicated = Vec::new();
    for mode in target_modes {
        if !deduplicated.contains(&mode) {
            deduplicated.push(mode);
        }
    }
    deduplicated
}

fn selection_update_report(
    plugin_id: &str,
    project_selection: ProjectPluginSelection,
    diagnostic: String,
) -> EditorPluginSelectionUpdateReport {
    EditorPluginSelectionUpdateReport {
        plugin_id: plugin_id.to_string(),
        project_selection,
        diagnostics: vec![diagnostic],
    }
}
