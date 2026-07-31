mod identity;
mod interfaces;
mod system_anchors;

#[cfg(test)]
mod tests;

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
        RuntimePluginPackageValidationProjection,
    },
    RuntimePluginDescriptor,
};

#[cfg(test)]
use super::super::package_validation::RuntimePluginPackageValidationMetrics;
#[cfg(test)]
use super::super::{
    feature_validation::{
        begin_feature_projection_build_observation, observed_embedded_feature_projection_views,
        observed_standalone_feature_projection_builds,
    },
    package_validation::{
        begin_package_projection_build_observation, observed_package_projection_builds,
    },
};

pub(in crate::plugin::runtime_plugin::registration_report) fn validate_runtime_plugin_package_manifest<
    'a,
>(
    descriptor: Option<&RuntimePluginDescriptor>,
    package_manifest: &'a PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) -> RuntimePluginPackageValidationProjection<'a> {
    let projection = RuntimePluginPackageValidationProjection::build(package_manifest);
    validate_runtime_plugin_registration_package_identity(
        descriptor,
        package_manifest,
        diagnostics,
    );
    validate_runtime_plugin_package_layout(package_manifest, &projection, diagnostics);
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
    validate_runtime_plugin_package_capabilities(package_manifest, &projection, diagnostics);
    validate_runtime_plugin_package_dependencies(package_manifest, &projection, diagnostics);
    validate_runtime_plugin_package_interfaces(package_manifest, &projection, diagnostics);
    validate_runtime_plugin_package_contributions(package_manifest, &projection, diagnostics);
    validate_runtime_plugin_package_asset_importers(package_manifest, &projection, diagnostics);
    validate_runtime_plugin_package_embedded_features(package_manifest, &projection, diagnostics);
    validate_runtime_plugin_package_capability_statuses(package_manifest, &projection, diagnostics);
    validate_runtime_plugin_package_modules(package_manifest, &projection, diagnostics);
    projection
}

#[cfg(test)]
fn validate_runtime_plugin_package_manifest_with_stats(
    descriptor: Option<&RuntimePluginDescriptor>,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) -> RuntimePluginPackageValidationMetrics {
    begin_package_projection_build_observation();
    begin_feature_projection_build_observation();
    let projection =
        validate_runtime_plugin_package_manifest(descriptor, package_manifest, diagnostics);
    let mut metrics = projection.metrics();
    metrics.projection_builds = observed_package_projection_builds();
    metrics.standalone_feature_projection_builds = observed_standalone_feature_projection_builds();
    metrics.embedded_feature_projection_views = observed_embedded_feature_projection_views();
    metrics
}
