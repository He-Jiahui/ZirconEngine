use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{plugin::NativePluginLoader, plugin::PluginModuleKind};

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
            let native_selection = native_project_selection(&package);
            if !completed
                .plugins
                .selections
                .iter()
                .any(|selection| selection.id == package.id)
            {
                completed.plugins.selections.push(native_selection);
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
                if selection.target_modes.is_empty() {
                    selection.target_modes = native_selection.target_modes.clone();
                }
                for native_feature in native_selection.features {
                    if let Some(existing_feature) = selection
                        .features
                        .iter_mut()
                        .find(|feature| feature.id == native_feature.id)
                    {
                        if existing_feature.runtime_crate.is_none() {
                            existing_feature.runtime_crate = native_feature.runtime_crate.clone();
                        }
                        if existing_feature.editor_crate.is_none() {
                            existing_feature.editor_crate = native_feature.editor_crate.clone();
                        }
                        if existing_feature.target_modes.is_empty() {
                            existing_feature.target_modes = native_feature.target_modes;
                        }
                    } else {
                        selection.features.push(native_feature);
                    }
                }
            }
        }
        completed
    }
}
