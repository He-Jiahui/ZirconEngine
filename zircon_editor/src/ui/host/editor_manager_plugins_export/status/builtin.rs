use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{
    ExportPackagingStrategy, PluginModuleKind, PluginPackageManifest, RuntimeTargetMode,
};

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::{module_crate, runtime_capabilities_for_package};
use super::super::reports::{EditorPluginStatus, EditorPluginStatusReport};

impl EditorManager {
    pub fn plugin_status_report(&self, manifest: &ProjectManifest) -> EditorPluginStatusReport {
        let runtime_catalog = self.runtime_plugin_catalog();
        let editor_catalog = self.editor_plugin_catalog();
        let packages = editor_catalog.package_manifests();
        let mut diagnostics = Vec::new();
        diagnostics.extend(runtime_catalog.diagnostics().iter().cloned());
        diagnostics.extend(editor_catalog.diagnostics().iter().cloned());

        let mut plugins = packages
            .iter()
            .map(|package| {
                let project_selection = manifest
                    .plugins
                    .selections
                    .iter()
                    .find(|selection| selection.id == package.id);
                let catalog_selection = runtime_catalog.project_selection_for_package(&package.id);
                let runtime_crate = project_selection
                    .and_then(|selection| selection.runtime_crate.clone())
                    .or_else(|| {
                        catalog_selection
                            .as_ref()
                            .and_then(|selection| selection.runtime_crate.clone())
                    })
                    .or_else(|| module_crate(package, PluginModuleKind::Runtime));
                let editor_crate = project_selection
                    .and_then(|selection| selection.editor_crate.clone())
                    .or_else(|| module_crate(package, PluginModuleKind::Editor));
                let runtime_capabilities = runtime_capabilities_for_package(package);
                let editor_capabilities = editor_catalog.capabilities_for_package(&package.id);
                let mut plugin_diagnostics = Vec::new();
                if package
                    .modules
                    .iter()
                    .any(|module| module.kind == PluginModuleKind::Runtime)
                    && runtime_crate.is_none()
                {
                    plugin_diagnostics.push("runtime crate is not declared".to_string());
                }
                if editor_crate.is_none() && !editor_capabilities.is_empty() {
                    plugin_diagnostics.push(
                        "editor capabilities are declared without an editor crate".to_string(),
                    );
                }
                EditorPluginStatus {
                    plugin_id: package.id.clone(),
                    display_name: package.display_name.clone(),
                    package_source: "builtin".to_string(),
                    load_state: "catalog".to_string(),
                    enabled: project_selection
                        .map(|selection| selection.enabled)
                        .unwrap_or(false),
                    required: project_selection
                        .map(|selection| selection.required)
                        .or_else(|| {
                            catalog_selection
                                .as_ref()
                                .map(|selection| selection.required)
                        })
                        .unwrap_or(false),
                    target_modes: project_selection
                        .map(|selection| selection.target_modes.clone())
                        .filter(|modes| !modes.is_empty())
                        .or_else(|| {
                            catalog_selection
                                .as_ref()
                                .map(|selection| selection.target_modes.clone())
                                .filter(|modes| !modes.is_empty())
                        })
                        .unwrap_or_else(|| target_modes_for_package(package)),
                    packaging: project_selection
                        .map(|selection| selection.packaging)
                        .or_else(|| {
                            catalog_selection
                                .as_ref()
                                .map(|selection| selection.packaging)
                        })
                        .unwrap_or_else(|| default_packaging_for_builtin_package(package)),
                    runtime_crate,
                    editor_crate,
                    runtime_capabilities,
                    editor_capabilities,
                    diagnostics: plugin_diagnostics,
                }
            })
            .collect::<Vec<_>>();
        plugins.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));

        EditorPluginStatusReport {
            plugins,
            diagnostics,
        }
    }
}

fn target_modes_for_package(package: &PluginPackageManifest) -> Vec<RuntimeTargetMode> {
    let mut modes = Vec::new();
    for mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    modes
}

fn default_packaging_for_builtin_package(
    package: &PluginPackageManifest,
) -> ExportPackagingStrategy {
    package
        .default_packaging
        .first()
        .copied()
        .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
}
