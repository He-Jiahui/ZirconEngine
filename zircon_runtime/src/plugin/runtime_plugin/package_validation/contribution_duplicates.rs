mod components;
mod event_catalogs;
mod options;
mod ui_components;

use crate::plugin::PluginPackageManifest;

pub(in crate::plugin::runtime_plugin) fn validate_duplicate_plugin_options(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    options::validate_duplicate_plugin_options(package_manifest, diagnostics);
}

pub(in crate::plugin::runtime_plugin) fn validate_duplicate_event_catalogs(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    event_catalogs::validate_duplicate_event_catalogs(package_manifest, diagnostics);
}

pub(in crate::plugin::runtime_plugin) fn validate_duplicate_components(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    components::validate_duplicate_components(package_manifest, diagnostics);
}

pub(in crate::plugin::runtime_plugin) fn validate_duplicate_ui_components(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    ui_components::validate_duplicate_ui_components(package_manifest, diagnostics);
}
