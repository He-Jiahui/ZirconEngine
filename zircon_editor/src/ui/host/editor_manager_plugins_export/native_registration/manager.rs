use std::path::Path;

use zircon_runtime::NativePluginLoader;

use crate::core::editor_plugin::EditorPluginRegistrationReport;

use super::super::super::editor_manager::EditorManager;
use super::registration_projection::{
    native_editor_registration_from_package, package_declares_editor_contribution,
};

impl EditorManager {
    pub fn native_editor_plugin_registration_reports(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Vec<EditorPluginRegistrationReport> {
        let native_report =
            NativePluginLoader.load_discovered_editor(self.plugin_directory(project_root));
        native_report
            .package_manifests()
            .into_iter()
            .filter(package_declares_editor_contribution)
            .map(|package| {
                let plugin_id = package.id.clone();
                native_editor_registration_from_package(
                    package,
                    native_report.diagnostics_for_editor_plugin(&plugin_id),
                )
            })
            .collect()
    }
}
