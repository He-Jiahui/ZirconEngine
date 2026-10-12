//! 清单诊断与清理的唯一组合入口；from_project_manifest 在生成包、注册代码前先执行这些约束。
#[path = "manifest/crates.rs"]
mod crates;
#[path = "manifest/duplicates.rs"]
mod duplicates;
#[path = "manifest/identity.rs"]
mod identity;
#[path = "manifest/projection.rs"]
mod projection;
#[path = "manifest/provider.rs"]
mod provider;
#[path = "manifest/target_modes.rs"]
mod target_modes;
#[path = "manifest/tokens.rs"]
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
