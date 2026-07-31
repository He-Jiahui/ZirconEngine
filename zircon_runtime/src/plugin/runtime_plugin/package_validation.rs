mod asset_importers;
mod capability_dependencies;
mod capability_status;
mod capability_status_references;
mod capability_status_targets;
mod contribution_duplicates;
mod contribution_owners;
mod contributions;
mod coordinates;
mod default_packaging;
mod embedded_feature_providers;
mod embedded_feature_targets;
mod embedded_features;
mod interfaces;
mod layout;
mod modules;
mod projection;
mod roots;
mod shape;
mod versions;

pub(in crate::plugin::runtime_plugin) use asset_importers::validate_runtime_plugin_package_asset_importers;
pub(in crate::plugin::runtime_plugin) use capability_dependencies::{
    validate_runtime_plugin_package_capabilities, validate_runtime_plugin_package_dependencies,
};
pub(in crate::plugin::runtime_plugin) use capability_status::validate_runtime_plugin_package_capability_statuses;
pub(in crate::plugin::runtime_plugin) use contributions::validate_runtime_plugin_package_contributions;
pub(in crate::plugin::runtime_plugin) use default_packaging::validate_runtime_plugin_default_packaging;
pub(in crate::plugin::runtime_plugin) use embedded_features::validate_runtime_plugin_package_embedded_features;
pub(in crate::plugin::runtime_plugin) use interfaces::validate_runtime_plugin_package_interfaces;
pub(in crate::plugin::runtime_plugin) use layout::validate_runtime_plugin_package_layout;
pub(in crate::plugin::runtime_plugin) use modules::validate_runtime_plugin_package_modules;
#[cfg(test)]
pub(in crate::plugin::runtime_plugin) use projection::{
    begin_package_projection_build_observation, observed_package_projection_builds,
};
pub(in crate::plugin::runtime_plugin) use projection::{
    EmbeddedFeatureKind, RuntimePluginPackageValidationMetrics,
    RuntimePluginPackageValidationProjection,
};
pub(in crate::plugin::runtime_plugin) use shape::{
    is_lowercase_runtime_plugin_token, validate_runtime_plugin_package_field,
    validate_runtime_plugin_package_id, validate_runtime_plugin_package_namespace,
    validate_runtime_plugin_package_token,
};
pub(in crate::plugin::runtime_plugin) use versions::validate_runtime_plugin_package_semver;
