use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginSelection;
use zircon_runtime::plugin::{PluginModuleKind, PluginPackageManifest};

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::{module_crate, project_selection_from_package};

#[cfg(test)]
#[path = "builtin/package_index_tests.rs"]
mod package_index_tests;

impl EditorManager {
    pub fn complete_project_plugin_manifest(&self, manifest: &ProjectManifest) -> ProjectManifest {
        let mut completed = manifest.clone();
        completed.plugins = Arc::unwrap_or_clone(
            self.runtime_plugin_catalog()
                .complete_project_manifest(&manifest.plugins, RuntimeTargetMode::EditorHost),
        );
        let editor_catalog = self.editor_plugin_catalog();
        let editor_packages = editor_catalog.package_manifests();
        complete_editor_package_selections(&mut completed.plugins.selections, editor_packages);
        completed
    }
}

fn complete_editor_package_selections(
    selections: &mut Vec<ProjectPluginSelection>,
    editor_packages: &[PluginPackageManifest],
) {
    let mut package_by_id = HashMap::with_capacity(editor_packages.len());
    for package in editor_packages {
        package_by_id.entry(package.id.as_str()).or_insert(package);
    }

    let mut package_presence = HashSet::with_capacity(editor_packages.len());
    for selection in selections.iter() {
        if let Some((package_id, _)) = package_by_id.get_key_value(selection.id.as_str()) {
            package_presence.insert(*package_id);
        }
    }
    for package in editor_packages {
        if package_presence.insert(package.id.as_str()) {
            selections.push(project_selection_from_package(package));
        }
    }

    for selection in selections {
        if selection.editor_crate.is_some() {
            continue;
        }
        selection.editor_crate = package_by_id
            .get(selection.id.as_str())
            .and_then(|package| module_crate(package, PluginModuleKind::Editor));
    }
}
