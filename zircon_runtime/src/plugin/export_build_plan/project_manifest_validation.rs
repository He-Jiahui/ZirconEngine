mod crates;
mod duplicates;
mod identity;
mod projection;
mod provider;
mod target_modes;
mod tokens;

pub(super) use crates::{
    project_editor_crate_diagnostics, project_runtime_crate_diagnostics,
    project_runtime_crate_name_is_valid, project_runtime_crate_override_is_valid,
    sanitize_invalid_project_crate_overrides,
};
pub(super) use duplicates::project_duplicate_selection_diagnostics;
pub(super) use identity::{
    project_feature_id_diagnostics, project_plugin_feature_id_is_valid,
    project_plugin_package_id_diagnostics, project_plugin_package_id_is_valid,
    sanitize_project_identity_rows,
};
pub(super) use projection::ProjectPluginManifestValidationProjection;
#[cfg(test)]
pub(super) use projection::{begin_projection_build_observation, observed_projection_builds};
pub(super) use provider::{
    project_feature_provider_package_id_diagnostics,
    sanitize_invalid_project_provider_package_overrides,
};
pub(super) use target_modes::{project_target_mode_diagnostics, sanitize_project_target_mode_rows};
