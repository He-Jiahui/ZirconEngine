use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::plugin::native::{discover_native_plugins, NativePluginLoadReport};
use zircon_runtime::{
    core::framework::project::ExportPackagingStrategy,
    core::framework::project::ProjectPluginSelection,
};

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
        let native_report = discover_native_plugins(self.plugin_directory(project_root.as_ref()));
        let mut selection = self.completed_native_aware_project_selection_from_load_report(
            manifest,
            plugin_id,
            &native_report,
        )?;
        selection.packaging = packaging;
        manifest.plugins.set_enabled(selection.clone());
        let completed =
            self.complete_project_plugin_manifest_with_native_report(manifest, &native_report);
        self.publish_project_plugin_status_from_load_report(&completed, &native_report);
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
        let native_report = discover_native_plugins(self.plugin_directory(project_root.as_ref()));
        let mut selection = self.completed_native_aware_project_selection_from_load_report(
            manifest,
            plugin_id,
            &native_report,
        )?;
        selection.target_modes = deduplicated_target_modes(target_modes);
        manifest.plugins.set_enabled(selection.clone());
        let completed =
            self.complete_project_plugin_manifest_with_native_report(manifest, &native_report);
        self.publish_project_plugin_status_from_load_report(&completed, &native_report);
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

    fn completed_native_aware_project_selection_from_load_report(
        &self,
        manifest: &ProjectManifest,
        plugin_id: &str,
        native_report: &NativePluginLoadReport,
    ) -> Result<ProjectPluginSelection, String> {
        self.complete_project_plugin_manifest_with_native_report(manifest, native_report)
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
