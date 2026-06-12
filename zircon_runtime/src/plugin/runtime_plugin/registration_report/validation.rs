mod identity;
mod interfaces;
mod system_anchors;

use crate::plugin::PluginPackageManifest;

use self::identity::validate_runtime_plugin_registration_package_identity;
pub(in crate::plugin::runtime_plugin::registration_report) use self::interfaces::validate_runtime_plugin_registration_interfaces;
pub(in crate::plugin::runtime_plugin::registration_report) use self::system_anchors::validate_runtime_plugin_registration_system_anchors;

use super::super::{
    package_validation::{
        validate_runtime_plugin_default_packaging, validate_runtime_plugin_package_asset_importers,
        validate_runtime_plugin_package_capabilities,
        validate_runtime_plugin_package_capability_statuses,
        validate_runtime_plugin_package_contributions,
        validate_runtime_plugin_package_dependencies,
        validate_runtime_plugin_package_embedded_features,
        validate_runtime_plugin_package_interfaces, validate_runtime_plugin_package_layout,
        validate_runtime_plugin_package_modules, validate_runtime_plugin_package_semver,
    },
    RuntimePluginDescriptor,
};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_package_manifest(
    descriptor: Option<&RuntimePluginDescriptor>,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_registration_package_identity(
        descriptor,
        package_manifest,
        diagnostics,
    );
    validate_runtime_plugin_package_layout(package_manifest, diagnostics);
    validate_runtime_plugin_package_semver("version", &package_manifest.version, diagnostics);
    validate_runtime_plugin_package_semver(
        "sdk_api_version",
        &package_manifest.sdk_api_version,
        diagnostics,
    );
    validate_runtime_plugin_default_packaging(
        "package manifest",
        &package_manifest.default_packaging,
        diagnostics,
    );
    validate_runtime_plugin_package_capabilities(package_manifest, diagnostics);
    validate_runtime_plugin_package_dependencies(package_manifest, diagnostics);
    validate_runtime_plugin_package_interfaces(package_manifest, diagnostics);
    validate_runtime_plugin_package_contributions(package_manifest, diagnostics);
    validate_runtime_plugin_package_asset_importers(package_manifest, diagnostics);
    validate_runtime_plugin_package_embedded_features(package_manifest, diagnostics);
    validate_runtime_plugin_package_capability_statuses(package_manifest, diagnostics);
    validate_runtime_plugin_package_modules(package_manifest, diagnostics);
}
