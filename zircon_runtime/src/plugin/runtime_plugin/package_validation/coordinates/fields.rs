use crate::plugin::PluginPackageManifest;

use super::{presence, shape};

pub(super) fn validate_runtime_plugin_package_coordinate_fields(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    if !presence::validate_runtime_plugin_package_coordinate_presence(package_manifest, diagnostics)
    {
        return;
    }

    shape::validate_runtime_plugin_package_coordinate_prefix(
        "package_prefix",
        &package_manifest.package_prefix,
        diagnostics,
    );
    shape::validate_runtime_plugin_package_coordinate_segment(
        "package_company",
        &package_manifest.package_company,
        diagnostics,
    );
    shape::validate_runtime_plugin_package_coordinate_segment(
        "package_name",
        &package_manifest.package_name,
        diagnostics,
    );
}
