use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::PluginModuleKind;

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::{module_crate, project_selection_from_package};

impl EditorManager {
    pub fn complete_project_plugin_manifest(&self, manifest: &ProjectManifest) -> ProjectManifest {
        let mut completed = manifest.clone();
        completed.plugins = self
            .runtime_plugin_catalog()
            .complete_project_manifest(&manifest.plugins);
        let editor_packages = self.editor_plugin_catalog().package_manifests();
        for package in &editor_packages {
            if completed
                .plugins
                .selections
                .iter()
                .any(|selection| selection.id == package.id)
            {
                continue;
            }
            completed
                .plugins
                .selections
                .push(project_selection_from_package(package));
        }
        for selection in &mut completed.plugins.selections {
            if selection.editor_crate.is_some() {
                continue;
            }
            selection.editor_crate = editor_packages
                .iter()
                .find(|package| package.id == selection.id)
                .and_then(|package| module_crate(package, PluginModuleKind::Editor));
        }
        completed
    }
}
