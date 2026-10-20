#[path = "validation/asset_importers.rs"]
mod asset_importers;
#[path = "validation/dependencies.rs"]
mod capability_dependencies;
#[path = "validation/capability_status.rs"]
mod capability_status;
#[path = "validation/references.rs"]
mod capability_status_references;
#[path = "validation/targets.rs"]
mod capability_status_targets;
#[path = "validation/duplicates.rs"]
mod contribution_duplicates;
#[path = "validation/contribution_owners.rs"]
mod contribution_owners;
#[path = "validation/contributions.rs"]
mod contributions;
#[path = "validation/coordinates.rs"]
mod coordinates;
#[path = "validation/default_packaging.rs"]
mod default_packaging;
#[path = "validation/embedded_feature_providers.rs"]
mod embedded_feature_providers;
#[path = "validation/embedded_feature_targets.rs"]
mod embedded_feature_targets;
#[path = "validation/embedded_features.rs"]
mod embedded_features;
#[path = "validation/interfaces.rs"]
mod interfaces;
#[path = "validation/layout.rs"]
mod layout;
#[path = "validation/modules.rs"]
mod modules;
#[path = "validation/projection.rs"]
mod projection;
#[path = "validation/roots.rs"]
mod roots;
#[path = "validation/shape.rs"]
mod shape;
#[path = "validation/versions.rs"]
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
pub(in crate::plugin::runtime_plugin) use projection::RuntimePluginPackageValidationMetrics;
#[cfg(test)]
pub(in crate::plugin::runtime_plugin) use projection::{
    begin_package_projection_build_observation, observed_package_projection_builds,
};
pub(in crate::plugin::runtime_plugin) use projection::{
    EmbeddedFeatureKind, RuntimePluginPackageValidationProjection,
};
pub(in crate::plugin::runtime_plugin) use shape::{
    is_lowercase_runtime_plugin_token, validate_runtime_plugin_package_field,
    validate_runtime_plugin_package_id, validate_runtime_plugin_package_namespace,
    validate_runtime_plugin_package_token,
};
pub(in crate::plugin::runtime_plugin) use versions::validate_runtime_plugin_package_semver;
