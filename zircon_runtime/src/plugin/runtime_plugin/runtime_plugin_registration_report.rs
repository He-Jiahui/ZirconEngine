use crate::core::ModuleDescriptor;
use crate::{
    ExportPackagingStrategy, PluginModuleKind, PluginPackageManifest, ProjectPluginSelection,
    RuntimeExtensionRegistry,
};

use super::RuntimePlugin;

#[derive(Clone, Debug)]
pub struct RuntimePluginRegistrationReport {
    pub package_manifest: PluginPackageManifest,
    pub project_selection: ProjectPluginSelection,
    pub extensions: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
}

impl RuntimePluginRegistrationReport {
    pub fn from_plugin(plugin: &dyn RuntimePlugin) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        if let Err(error) = plugin.register_runtime_extensions(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        Self {
            package_manifest: plugin.package_manifest(),
            project_selection: plugin.project_selection(),
            extensions,
            diagnostics,
        }
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn from_native_package_manifest(package_manifest: PluginPackageManifest) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        for module in package_manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
        {
            if let Err(error) = extensions.register_module(ModuleDescriptor::new(
                module.name.clone(),
                format!(
                    "Native dynamic runtime plugin module provided by {}",
                    package_manifest.id
                ),
            )) {
                diagnostics.push(error.to_string());
            }
        }
        Self {
            project_selection: native_project_selection_from_package(&package_manifest),
            package_manifest,
            extensions,
            diagnostics,
        }
    }
}

fn native_project_selection_from_package(
    package_manifest: &PluginPackageManifest,
) -> ProjectPluginSelection {
    let mut target_modes = Vec::new();
    for target_mode in package_manifest
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    ProjectPluginSelection {
        id: package_manifest.id.clone(),
        enabled: true,
        required: false,
        target_modes,
        packaging: ExportPackagingStrategy::NativeDynamic,
        runtime_crate: package_manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Runtime)
            .map(|module| module.crate_name.clone()),
        editor_crate: package_manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .map(|module| module.crate_name.clone()),
    }
}
