use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{NativePluginLoader, PluginModuleKind};

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::{module_crate, native_project_selection};

impl EditorManager {
    pub fn complete_native_aware_project_plugin_manifest(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
    ) -> ProjectManifest {
        let mut completed = self.complete_project_plugin_manifest(manifest);
        let native_report = NativePluginLoader.discover(self.plugin_directory(project_root));
        for package in native_report.package_manifests() {
            if !completed
                .plugins
                .selections
                .iter()
                .any(|selection| selection.id == package.id)
            {
                completed
                    .plugins
                    .selections
                    .push(native_project_selection(&package));
                continue;
            }
            if let Some(selection) = completed
                .plugins
                .selections
                .iter_mut()
                .find(|selection| selection.id == package.id)
            {
                if selection.runtime_crate.is_none() {
                    selection.runtime_crate = module_crate(&package, PluginModuleKind::Runtime);
                }
                if selection.editor_crate.is_none() {
                    selection.editor_crate = module_crate(&package, PluginModuleKind::Editor);
                }
            }
        }
        completed
    }
}
