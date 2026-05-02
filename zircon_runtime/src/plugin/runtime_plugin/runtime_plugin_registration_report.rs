use crate::core::ModuleDescriptor;
use crate::{
    plugin::ExportPackagingStrategy, plugin::PluginModuleKind, plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry,
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
        let package_manifest = plugin.package_manifest();
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        Self {
            package_manifest,
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
        for importer in package_manifest.asset_importers.iter().cloned() {
            if let Err(error) = extensions.register_asset_importer_descriptor(importer) {
                diagnostics.push(error.to_string());
            }
        }
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        Self {
            project_selection: native_project_selection_from_package(&package_manifest),
            package_manifest,
            extensions,
            diagnostics,
        }
    }
}

fn register_package_manifest_contributions(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for option in package_manifest.options.iter().cloned() {
        if extensions
            .plugin_options()
            .iter()
            .any(|existing| existing.key == option.key)
        {
            continue;
        }
        if let Err(error) = extensions.register_plugin_option(option) {
            diagnostics.push(error.to_string());
        }
    }
    for event_catalog in package_manifest.event_catalogs.iter().cloned() {
        if extensions
            .plugin_event_catalogs()
            .iter()
            .any(|existing| existing.namespace == event_catalog.namespace)
        {
            continue;
        }
        if let Err(error) = extensions.register_plugin_event_catalog(event_catalog) {
            diagnostics.push(error.to_string());
        }
    }
    for component in package_manifest.components.iter().cloned() {
        if extensions
            .components()
            .iter()
            .any(|existing| existing.type_id == component.type_id)
        {
            continue;
        }
        if let Err(error) = extensions.register_component(component) {
            diagnostics.push(error.to_string());
        }
    }
    for ui_component in package_manifest.ui_components.iter().cloned() {
        if extensions
            .ui_components()
            .iter()
            .any(|existing| existing.component_id == ui_component.component_id)
        {
            continue;
        }
        if let Err(error) = extensions.register_ui_component(ui_component) {
            diagnostics.push(error.to_string());
        }
    }
    for importer in package_manifest.asset_importers.iter().cloned() {
        if extensions
            .asset_importers()
            .descriptors()
            .iter()
            .any(|existing| existing.id == importer.id)
        {
            continue;
        }
        if let Err(error) = extensions.register_asset_importer_descriptor(importer) {
            diagnostics.push(error.to_string());
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
        features: Vec::new(),
    }
}
