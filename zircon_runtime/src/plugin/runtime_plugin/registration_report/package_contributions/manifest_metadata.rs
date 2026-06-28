use crate::plugin::{
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};

pub(super) fn register_package_manifest_metadata_contributions(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    register_package_manifest_options(package_manifest, extensions, diagnostics);
    register_package_manifest_event_catalogs(package_manifest, extensions, diagnostics);
    register_package_manifest_components(package_manifest, extensions, diagnostics);
    register_package_manifest_ui_components(package_manifest, extensions, diagnostics);
    register_package_manifest_geometry_sources(package_manifest, extensions, diagnostics);
    register_package_manifest_shading_models(package_manifest, extensions, diagnostics);
}

fn register_package_manifest_options(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for option in package_manifest.options.iter().cloned() {
        match extensions.register_plugin_option(option) {
            Ok(()) | Err(RuntimeExtensionRegistryError::DuplicatePluginOption(_)) => {}
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}

fn register_package_manifest_event_catalogs(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for event_catalog in package_manifest.event_catalogs.iter().cloned() {
        match extensions.register_plugin_event_catalog(event_catalog) {
            Ok(()) | Err(RuntimeExtensionRegistryError::DuplicatePluginEventCatalog(_)) => {}
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}

fn register_package_manifest_components(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for component in package_manifest.components.iter().cloned() {
        match extensions.register_component(component) {
            Ok(()) | Err(RuntimeExtensionRegistryError::DuplicateComponentType(_)) => {}
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}

fn register_package_manifest_ui_components(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for ui_component in package_manifest.ui_components.iter().cloned() {
        match extensions.register_ui_component(ui_component) {
            Ok(()) | Err(RuntimeExtensionRegistryError::DuplicateUiComponent(_)) => {}
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}

fn register_package_manifest_geometry_sources(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for descriptor in package_manifest.geometry_sources.iter().cloned() {
        match extensions.register_geometry_source(&package_manifest.id, descriptor) {
            Ok(()) | Err(RuntimeExtensionRegistryError::DuplicateGeometrySource(_)) => {}
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}

fn register_package_manifest_shading_models(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    for descriptor in package_manifest.shading_models.iter().cloned() {
        match extensions.register_shading_model(&package_manifest.id, descriptor) {
            Ok(()) | Err(RuntimeExtensionRegistryError::DuplicateShadingModel(_)) => {}
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}
