mod components;
mod event_catalogs;
mod ui_components;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_event_catalog_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    event_catalogs::validate_event_catalog_owners(package_manifest, diagnostics);
}

pub(in crate::plugin::runtime_plugin) fn validate_component_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    components::validate_component_owners(package_manifest, diagnostics);
}

pub(in crate::plugin::runtime_plugin) fn validate_ui_component_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    ui_components::validate_ui_component_owners(package_manifest, diagnostics);
}
