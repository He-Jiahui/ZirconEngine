use crate::plugin::PluginPackageManifest;

use super::super::contribution_duplicates::{
    validate_duplicate_components, validate_duplicate_event_catalogs,
    validate_duplicate_plugin_options, validate_duplicate_ui_components,
};
use super::super::contribution_owners::{
    validate_component_owners, validate_event_catalog_owners, validate_ui_component_owners,
};

pub(super) fn validate_runtime_plugin_package_contribution_groups(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_duplicate_plugin_options(package_manifest, diagnostics);
    validate_duplicate_event_catalogs(package_manifest, diagnostics);
    validate_duplicate_components(package_manifest, diagnostics);
    validate_duplicate_ui_components(package_manifest, diagnostics);
    validate_event_catalog_owners(package_manifest, diagnostics);
    validate_component_owners(package_manifest, diagnostics);
    validate_ui_component_owners(package_manifest, diagnostics);
}
